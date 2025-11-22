use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::env;
use std::io::Result;

use handlers::{auth, poker_session};
use middleware::AuthMiddleware;
use utils::establish_connection_pool;

use crate::handlers;
use crate::middleware;
use crate::utils;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub(crate) struct PokerTrackerApp;

impl PokerTrackerApp {
    pub fn new() -> Self {
        PokerTrackerApp
    }

    pub async fn run(self) -> Result<()> {
        let pool = establish_connection_pool();

        // Run migrations
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let bind_address = format!("{}:{}", host, port);

        log::info!("Starting server at http://{}", bind_address);

        HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            App::new()
                .wrap(cors)
                .app_data(web::Data::new(pool.clone()))
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/auth")
                                .route("/register", web::post().to(auth::register))
                                .route("/login", web::post().to(auth::login))
                                .service(
                                    web::scope("")
                                        .wrap(AuthMiddleware)
                                        .route("/me", web::get().to(auth::get_me))
                                        .route(
                                            "/cookie-consent",
                                            web::put().to(auth::update_cookie_consent),
                                        )
                                        .route(
                                            "/change-password",
                                            web::post().to(auth::change_password),
                                        ),
                                ),
                        )
                        .service(
                            web::scope("/sessions")
                                .wrap(AuthMiddleware)
                                .route("", web::post().to(poker_session::create_session))
                                .route("", web::get().to(poker_session::get_sessions))
                                .route("/{id}", web::get().to(poker_session::get_session))
                                .route("/{id}", web::put().to(poker_session::update_session))
                                .route("/{id}", web::delete().to(poker_session::delete_session)),
                        ),
                )
        })
        .bind(&bind_address)?
        .run()
        .await
    }
}
