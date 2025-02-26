#![allow(dead_code)]
use std::sync::Arc;

use async_trait::async_trait;
use songbird::{events::context_data::{ConnectData, DisconnectData, RtpData, VoiceTick}, model::payload::{ClientDisconnect, Speaking}, Event, EventContext, EventHandler};
use tokio::sync::Mutex;

pub struct DriverEventHandler {
    
}

impl DriverEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn on_client_disconnect(&self, data: &ClientDisconnect) {
        let _ = data;
    }

    pub async fn on_voice_tick(&self, data: &VoiceTick) {
        let _ = data;
    }

    pub async fn on_speaking_state_update(&self, data: &Speaking) {
        let _ = data;
    }

    pub async fn on_rtp_packet(&self, data: &RtpData) {
        let _ = data;
    }

    pub async fn on_driver_connect(&self, data: &ConnectData<'_>) {
        let _ = data;
    }

    pub async fn on_driver_disconnect(&self, data: &DisconnectData<'_>) {
        let _ = data;
    }
}

#[derive(Clone)]
pub struct DriverCallback {
    event_handler: Arc<Mutex<DriverEventHandler>>,
}

impl DriverCallback {
    pub fn new(event_handler: DriverEventHandler) -> Self {
        Self {
            event_handler: Arc::new(Mutex::new(event_handler)),
        }
    }
}

#[async_trait]
impl EventHandler for DriverCallback {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let handler = self.event_handler.lock().await;
        match ctx {
            EventContext::ClientDisconnect(data) => handler.on_client_disconnect(data).await,
            EventContext::VoiceTick(data) => handler.on_voice_tick(data).await,
            EventContext::SpeakingStateUpdate(data) => handler.on_speaking_state_update(data).await,
            EventContext::RtpPacket(data) => handler.on_rtp_packet(data).await,
            EventContext::DriverConnect(data) => handler.on_driver_connect(data).await,
            EventContext::DriverDisconnect(data) => handler.on_driver_disconnect(data).await,
            _ => (),
        };
        None
    }
}
