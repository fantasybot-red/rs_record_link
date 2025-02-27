use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    num::NonZero,
    sync::Arc,
};

use async_trait::async_trait;
use hound::{WavSpec, WavWriter};
use songbird::{
    events::context_data::{ConnectData, DisconnectData, VoiceData, VoiceTick},
    id::UserId,
    model::payload::ClientDisconnect,
    Event, EventContext, EventHandler,
};
use tokio::sync::Mutex;
use tokio::{sync::mpsc::UnboundedSender, task};

use super::WebSocketMessage;

pub struct DriverEventHandler {
    senders: UnboundedSender<WebSocketMessage>,
    recording_config: WavSpec,
    bot_id: Option<UserId>,
    is_recording: bool,
    recording_limit: Option<usize>,
    recorder: Option<WavWriter<BufWriter<File>>>,
    destroy: bool,
}

impl DriverEventHandler {
    fn new_recorder<T: Write + Seek>(&self, file: T) -> WavWriter<BufWriter<T>> {
        let spec = self.recording_config.clone();
        let buffer = BufWriter::new(file);
        let writer = WavWriter::new(buffer, spec).unwrap();
        writer
    }

    pub fn new(sender: UnboundedSender<WebSocketMessage>) -> Arc<Mutex<Self>> {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let new_self = Self {
            senders: sender,
            recording_config: spec,
            bot_id: None,
            is_recording: false,
            recording_limit: None,
            recorder: None,
            destroy: false,
        };
        Arc::new(Mutex::new(new_self))
    }

    pub fn set_bot_id(&mut self, bot_id: UserId) {
        self.bot_id = Some(bot_id);
    }

    pub fn start_recording(&mut self) {
        let file = File::create("output.wav").unwrap();
        let recorder = self.new_recorder(file);
        self.recorder = Some(recorder);
        self.is_recording = true;
        self.senders
            .send(WebSocketMessage::new_event("RECORDING_STARTED"))
            .unwrap();
    }

    pub fn stop_recording(&mut self) {
        if !self.is_recording {
            return;
        }
        self.is_recording = false;
        if let Some(recorder) = self.recorder.take() {
            let _ = recorder.finalize();
        }
        self.recorder = None;
        self.senders
            .send(WebSocketMessage::new_event("RECORDING_STOPPED"))
            .unwrap();
    }

    pub fn add_emtpy_frame(&mut self) {
        if let Some(ref mut recorder) = self.recorder {
            let rconfig = self.recording_config.clone();
            let duration_ms = 20; // songbird tick duration
            let len_samples = (rconfig.sample_rate * duration_ms / 1000) * rconfig.channels as u32;
            let silence = vec![0; len_samples as usize];
            for sample in silence.iter() {
                recorder.write_sample(*sample).unwrap();
            }
        }
    }

    pub fn merge_audio(data: Vec<Vec<i16>>) -> Vec<i16> {
        if data.is_empty() {
            return vec![];
        }

        let num_channels = data[0].len();
        let mut mixed_audio = vec![0i16; num_channels];

        for frame in data.iter() {
            for (i, sample) in frame.iter().enumerate() {
                mixed_audio[i] = mixed_audio[i].saturating_add(*sample);
            }
        }

        mixed_audio
    }

    pub async fn on_voice_tick(&mut self, data: &VoiceTick) {
        if !self.is_recording {
            return;
        }
        let ative = &data.speaking.values().collect::<Vec<&VoiceData>>();
        if ative.len() == 0 {
            return self.add_emtpy_frame();
        }
        let audios = ative
            .iter()
            .filter(|x| x.decoded_voice.is_some())
            .map(|x| x.decoded_voice.as_ref().unwrap().clone())
            .collect::<Vec<Vec<i16>>>();
        let audio_merge = task::spawn_blocking(move || Self::merge_audio(audios))
            .await
            .unwrap();
        if let Some(ref mut recorder) = self.recorder {
            for sample in audio_merge {
                recorder.write_sample(sample).unwrap();
            }
            
            recorder.flush().unwrap();

            if let Some(limit) = self.recording_limit {
                if recorder.len() as usize >= limit {
                    self.stop_recording();
                }
            }
        }
    }

    pub async fn on_client_disconnect(&mut self, data: &ClientDisconnect) {
        if let Some(bot_id) = self.bot_id {
            if bot_id.0 == NonZero::new(data.user_id.0).unwrap() {
                let msg = WebSocketMessage::new_event("DISCONNECTED");
                let _ = self.senders.send(msg);
                self.stop_recording();
                self.destroy = true;
            }
        }
    }

    pub async fn on_driver_connect(&self, _: &ConnectData<'_>) {
        let msg = WebSocketMessage::new_event("CONNECTED");
        let _ = self.senders.send(msg);
    }

    pub async fn on_driver_disconnect(&mut self, _: &DisconnectData<'_>) {
        let msg = WebSocketMessage::new_event("DISCONNECTED");
        let _ = self.senders.send(msg);
        self.stop_recording();
        self.destroy = true;
    }
}

#[derive(Clone)]
pub struct DriverCallback {
    event_handler: Arc<Mutex<DriverEventHandler>>,
}

impl DriverCallback {
    pub fn new(event_handler: Arc<Mutex<DriverEventHandler>>) -> Self {
        Self {
            event_handler: event_handler,
        }
    }
}

#[async_trait]
impl EventHandler for DriverCallback {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut handler = self.event_handler.lock().await;
        match ctx {
            EventContext::ClientDisconnect(data) => handler.on_client_disconnect(data).await,
            EventContext::VoiceTick(data) => handler.on_voice_tick(data).await,
            EventContext::DriverConnect(data) => handler.on_driver_connect(data).await,
            EventContext::DriverDisconnect(data) => handler.on_driver_disconnect(data).await,
            _ => (),
        };
        drop(handler);
        None
    }
}
