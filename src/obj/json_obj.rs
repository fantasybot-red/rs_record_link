#![allow(unused)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use songbird::{driver, id::{GuildId, UserId}};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketMessage {
    t: String,
    d: Value,
}

impl WebSocketMessage {

    pub fn new<T: Serialize>(t: String, d: T) -> Self {
        WebSocketMessage {
            t,
            d: serde_json::to_value(d).unwrap()
        }
    }

    pub fn get_event_name(&self) -> &str {
        &self.t
    }

    pub fn voice_server_update(&self) -> Result<VoiceServerUpdate, serde_json::Error> {
        serde_json::from_value(self.d.clone())
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoiceServerUpdate {
    endpoint: String,
    guild_id: String,
    token: String,
    user_id: String,
    session_id: String
}

impl VoiceServerUpdate {
    pub fn to_connection_info(&self) -> songbird::ConnectionInfo {
        songbird::ConnectionInfo {
            channel_id: None,
            endpoint: self.endpoint.clone(),
            guild_id: GuildId(self.guild_id.parse().unwrap()),
            token: self.token.clone(),
            session_id: self.session_id.clone(),
            user_id: UserId(self.user_id.parse().unwrap()),
        }
    }
}