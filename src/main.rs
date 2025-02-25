mod helper;
mod obj;
mod middleware;
mod router;

use axum::{
    middleware::from_fn_with_state, Router
};

use tower_http::trace::TraceLayer;


#[tokio::main]
async fn main() {

    helper::setup_logger();

    helper::load_dotenv();

    let config_r = obj::Config::load_config();

    if config_r.is_err() {
        eprintln!("ERR: Failed to load config: {:?}", config_r.err());
        std::process::exit(1);
    }

    let config = config_r.unwrap();

    let app = Router::new()
        .merge(router::export_router())
        .layer(from_fn_with_state(config.clone(), middleware::auth))
        .layer(TraceLayer::new_for_http())
        .with_state(config.clone());

    let bind_addr = config.bind_addr;

    println!("Listening on: {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(helper::shutdown_signal())
        .await
        .unwrap();
}

