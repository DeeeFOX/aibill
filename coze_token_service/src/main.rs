use coze_token_service::config;
use coze_token_service::routes::routing::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing with environment-based default level
    let default_level = if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
        "info"
    } else {
        "debug"
    };
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Coze Token Service");
    info!("Loading configuration...");

    let config = config::config::load_config();
    info!("Configuration loaded successfully");
    
    info!("Creating router...");
    let app = create_router().with_state(config.clone());
    info!("Router created successfully");

    info!("Binding TCP listener on 0.0.0.0:9000...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("Failed to bind TCP listener");
    
    info!("Successfully listening on {}", listener.local_addr().unwrap());
    info!("Server is ready to accept connections");

    axum::serve(listener, app)
        .await
        .unwrap();
}
