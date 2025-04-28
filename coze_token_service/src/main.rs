use coze_token_service::config;
use coze_token_service::routes::routing::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Coze Token Service");

    let config = config::config::load_config();
    let app = create_router().with_state(config.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000") // Assuming a default server address or get from env if needed
        .await
        .unwrap();
    
    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .unwrap();
}
