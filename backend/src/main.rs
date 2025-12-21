mod app;
mod handlers;
mod middleware;
mod models;
mod schema;
mod utils;

use std::io::Result;

use dotenvy::dotenv;

use crate::app::PokerTrackerApp;

#[tokio::main(flavor = "multi_thread", worker_threads = 100)]
async fn main() -> Result<()> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let app = PokerTrackerApp::new();
    app.run().await
}
