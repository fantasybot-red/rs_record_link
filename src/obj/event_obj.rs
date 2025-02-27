#![allow(dead_code)]
use std::{
    fs::File, io::{BufWriter, Seek, Write}, num::NonZero, sync::Arc
};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::task;
use hound::{WavSpec, WavWriter};
use songbird::model::id::UserId as VoiceUserId;
use songbird::{
    events::context_data::{ConnectData, DisconnectData, RtpData, VoiceData, VoiceTick},
    id::UserId,
    model::payload::{ClientDisconnect, Speaking},
    Event, EventContext, EventHandler,
};
use tokio::sync::Mutex;

pub struct DriverEventHandler {
    recording_config: WavSpec,
    bot_id: Option<UserId>,
    is_recording: bool,
    ssrcs: DashMap<u32, VoiceUserId>,
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

    pub fn new() -> Arc<Mutex<Self>> {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let new_self = Self {
            recording_config: spec,
            bot_id: None,
            is_recording: false,
            ssrcs: DashMap::new(),
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
    }

    pub fn stop_recording(&mut self) {
        self.is_recording = false;
        if let Some(recorder) = self.recorder.take() {
            let _ = recorder.finalize();
        }
        self.recorder = None;
    }

    pub fn add_emtpy_frame(&mut self) {
        if let Some(ref mut recorder) = self.recorder {
            let rconfig = self.recording_config.clone();
            let len_audio = (rconfig.sample_rate * rconfig.channels as u32) as usize;
            let silence = vec![0; len_audio];
            for sample in silence.iter() {
                recorder.write_sample(*sample).unwrap();
            }
        }
    }

    pub fn merge_audio(data: Vec<Vec<i16>>) -> Vec<i16> {
    
        let max_len = data.iter().map(|d| d.len()).max().unwrap_or(0);
        let mut mixed_audio = vec![0i32; max_len]; // Use i32 to prevent overflow
    
        // Mix audio samples
        for buffer in &data {
            for (i, &sample) in buffer.iter().enumerate() {
                mixed_audio[i] += sample as i32;
            }
        }
    
        // Normalize to prevent clipping
        let mixed_audio: Vec<i16> = mixed_audio
            .into_iter()
            .map(|sample| sample.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
            .collect();
    
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
        let audio_merge = task::spawn_blocking(move || Self::merge_audio(audios)).await.unwrap();
        if let Some(ref mut recorder) = self.recorder {
            for sample in audio_merge {
                recorder.write_sample(sample).unwrap();
            }
            if (recorder.duration() % 2 == 0) && (recorder.duration() > 0) {
                let _ = recorder.flush();
            }
        }
    }

    pub async fn on_speaking_state_update(&self, data: &Speaking) {
        let ssrc = data.ssrc;
        let user_id_r = data.user_id;
        if user_id_r.is_none() {
            return;
        }
        let user_id = user_id_r.unwrap();
        self.ssrcs.insert(ssrc, user_id);
    }

    pub async fn on_rtp_packet(&self, data: &RtpData) {
        let _ = data;
        // todo capcher video
    }

    pub async fn on_client_disconnect(&mut self, data: &ClientDisconnect) {
        if let Some(bot_id) = self.bot_id {
            if bot_id.0 == NonZero::new(data.user_id.0).unwrap() {
                self.stop_recording();
                self.destroy = true;
            }
        }
    }

    pub async fn on_driver_connect(&self, data: &ConnectData<'_>) {
        let _ = data;
    }

    pub async fn on_driver_disconnect(&mut self, _: &DisconnectData<'_>) {
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
            EventContext::SpeakingStateUpdate(data) => handler.on_speaking_state_update(data).await,
            EventContext::RtpPacket(data) => handler.on_rtp_packet(data).await,
            EventContext::DriverConnect(data) => handler.on_driver_connect(data).await,
            EventContext::DriverDisconnect(data) => handler.on_driver_disconnect(data).await,
            _ => (),
        };
        drop(handler);
        None
    }
}
