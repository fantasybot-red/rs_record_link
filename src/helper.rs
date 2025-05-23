use std::fs::exists as file_exists;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub fn load_dotenv() {
    let check_r = file_exists(".env");
    if check_r.is_ok() {
        if check_r.unwrap() {
            println!("Load .env file");
            let env_loader = dotenvy::dotenv();
            if env_loader.is_ok() {
                println!("Loaded .env file");
            } else {
                eprintln!("ERR: Failed to load .env file: {:?}", env_loader.err());
                std::process::exit(1);
            }
        }
    }
}

pub fn setup_logger() {
    let env_filter = EnvFilter::from_default_env()
        .add_directive("tower_http=debug".parse().unwrap())
        .add_directive("axum=trace".parse().unwrap()); // use for tracing tower and axum logs
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}