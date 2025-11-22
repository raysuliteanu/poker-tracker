mod app;
mod handlers;
mod middleware;
mod models;
mod schema;
mod utils;

use dotenvy::dotenv;

use crate::app::PokerApp;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let app = PokerApp::new();
    app.run().await
}
