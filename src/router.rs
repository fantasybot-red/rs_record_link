use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::State;
use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use axum::routing::*;
use axum::Router;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use futures_util::StreamExt as _;
use songbird::{Driver, Config as SongbirdConfig};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::obj::Config;
use crate::obj::WebSocketMessage;

pub fn export_router() -> Router<Config> {
    let router = Router::new()
    .route("/", get(handler));
    router
}

async fn handler(ws: WebSocketUpgrade, State(config): State<Config>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, config))
}

async fn send_task(mut stx: SplitSink<WebSocket, Message>, mut rx_s: UnboundedReceiver<WebSocketMessage>) {
    tokio::spawn(async move {
        while let Some(msg) = rx_s.recv().await {
            let msg = Message::Text(serde_json::to_string(&msg).unwrap().into());
            let _ = stx.send(msg).await;
        }
    });
}

async fn handle_socket(socket: WebSocket, _config: Config) {
    let (stx, mut srx) = socket.split();
    let (sender, rx_s) = tokio::sync::mpsc::unbounded_channel::<WebSocketMessage>();
    send_task(stx, rx_s).await;
    let sb_config  = SongbirdConfig::default();
    let mut driver = Driver::new(sb_config);
    while let Some(Ok(msg)) = srx.next().await {
        let text_r = msg.to_text();
        if !text_r.is_err() { drop(sender.clone()); };
        let text = text_r.unwrap();
        let json_r = serde_json::from_str::<WebSocketMessage>(text);
        if json_r.is_err() { drop(sender.clone()); };
        let json = json_r.unwrap();


        if json.get_event_name() == "VOICE_SERVER_UPDATE" {
            let voice_server_update = json.voice_server_update().unwrap();
            let connection_info = voice_server_update.to_connection_info();
            driver.connect(connection_info).await.unwrap();
        } else {
            drop(sender.clone());
        }
    }
}
