#![allow(unused)]
use serde::{de::Error, Deserialize, Serialize};
use serde_json::Value;
use songbird::{driver, id::{GuildId, UserId}};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketMessage {
    t: String,
    d: Option<Value>,
}

impl WebSocketMessage {

    pub fn new<T: Serialize, S: ToString>(t: S, d: T) -> Self {
        WebSocketMessage {
            t: t.to_string(),
            d: Some(serde_json::to_value(d).unwrap())
        }
    }

    pub fn new_event<T: ToString>(t: T) -> Self {
        WebSocketMessage {
            t: t.to_string(),
            d: None
        }
    }

    pub fn gen(&self) -> &str {
        &self.t
    }

    pub fn voice_server_update(&self) -> Result<VoiceServerUpdate, serde_json::Error> {
        let data_r = self.d.clone();
        if data_r.is_none() {
            return Err(serde_json::Error::custom("No data"));
        }
        let data = data_r.unwrap();
        serde_json::from_value(data)
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