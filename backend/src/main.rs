mod app;
mod handlers;
mod middleware;
mod models;
mod schema;
mod utils;

use std::io::Result;

use dotenvy::dotenv;

use crate::app::PokerTrackerApp;
use crate::utils::AppConfig;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    dotenv().ok(); // Still support .env for backward compatibility

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration (TOML + env overrides + defaults)
    let config = match AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Required: DATABASE_URL and JWT_SECRET must be set via environment or poker-tracker.toml");
            std::process::exit(1);
        }
    };

    let app = PokerTrackerApp::new(config);
    app.run().await
}
