mod app;
mod handlers;
mod middleware;
mod models;
mod schema;
mod utils;

use std::io::Result;

use dotenvy::dotenv;

use crate::app::PokerTrackerApp;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let app = PokerTrackerApp::new();
    app.run().await
}
