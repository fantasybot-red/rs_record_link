use songbird::{Event, EventContext, EventHandler};
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;
use super::WebSocketMessage;

#[derive(Clone)]
pub struct CallbackR {
    ws: UnboundedSender<WebSocketMessage>,
}

impl CallbackR {
    pub fn new(ws: UnboundedSender<WebSocketMessage>) -> Self {
        Self { ws }
    }
}

#[async_trait]
impl EventHandler for CallbackR {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            EventContext::ClientDisconnect(data) => {
                
            }
            EventContext::VoiceTick(data) => {
                
            }
            EventContext::SpeakingStateUpdate(data) => {
                
            }
            EventContext::RtpPacket(data) => {
                
            }
            EventContext::DriverConnect(data) => {
                
            }
            _ => {
                // We won't be registering this struct for any more event classes.
                unimplemented!()
            }
        }
        None
    }
}
