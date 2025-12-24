use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use handlers::{auth, poker_session};
use middleware::AuthLayer;
use utils::establish_connection_pool;

use diesel::RunQueryDsl;
use diesel::sql_types::Integer;

use crate::utils::PokerTrackerConfig;
use crate::{handlers, middleware, utils};

// this method is called from the /api/health route, via Axum
async fn health(State(state): State<Arc<AppState>>) -> Response {
    if let Ok(mut conn) = state.db_provider.get_connection()
        && let Ok(_) = diesel::select(diesel::dsl::sql::<Integer>("1")).execute(&mut conn)
    {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "Ok"
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database connection failed"
            })),
        )
            .into_response()
    }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Shared application state
pub struct AppState {
    pub db_provider: Arc<dyn utils::DbProvider>,
    pub config: PokerTrackerConfig,
}

/// Create the application router with the given state.
pub fn create_app_router(state: Arc<AppState>) -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(3600));

    let jwt_secret = state.config.jwt_secret.clone();

    Router::new()
        .route("/api/health", get(health))
        // Public auth routes
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        // Protected auth routes
        .route("/api/auth/me", get(auth::get_me))
        .route("/api/auth/cookie-consent", put(auth::update_cookie_consent))
        .route("/api/auth/change-password", post(auth::change_password))
        // Protected session routes
        .route(
            "/api/sessions",
            post(poker_session::create_session).get(poker_session::get_sessions),
        )
        .route("/api/sessions/export", get(poker_session::export_sessions))
        .route(
            "/api/sessions/{id}",
            get(poker_session::get_session)
                .put(poker_session::update_session)
                .delete(poker_session::delete_session),
        )
        // Apply middleware
        .layer(AuthLayer::new(jwt_secret))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub struct PokerTrackerApp {
    config: PokerTrackerConfig,
}

impl PokerTrackerApp {
    pub fn new(config: PokerTrackerConfig) -> Self {
        PokerTrackerApp { config }
    }

    pub async fn run(self) -> std::io::Result<()> {
        let pool = establish_connection_pool(&self.config);

        // Run migrations
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        let bind_address = format!("{}:{}", self.config.host, self.config.port);

        tracing::info!("Starting server at http://{}", bind_address);

        // Create shared application state
        let state = Arc::new(AppState {
            db_provider: Arc::new(pool),
            config: self.config.clone(),
        });

        // Build the router using the extracted function
        let app = create_app_router(state);

        // Parse bind address
        let addr: SocketAddr = bind_address
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        // Create TCP listener
        let listener = tokio::net::TcpListener::bind(addr).await?;

        // Run server
        axum::serve(listener, app)
            .await
            .map_err(std::io::Error::other)
    }
}
