use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PokerTrackerConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String, // Required, no default
    #[serde(default = "default_max_connections")]
    pub maxconnections: u32,
    #[serde(default = "default_min_idle")]
    pub minidle: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub jwtsecret: String, // Required, no default
    #[serde(default = "default_bcrypt_cost")]
    pub bcryptcost: u32,
}

// Default value functions
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_max_connections() -> u32 {
    100
}

fn default_min_idle() -> u32 {
    10
}

fn default_bcrypt_cost() -> u32 {
    bcrypt::DEFAULT_COST
}

impl PokerTrackerConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // Start with defaults
            .set_default("server.host", default_host())?
            .set_default("server.port", default_port() as i64)?
            .set_default("database.maxconnections", default_max_connections() as i64)?
            .set_default("database.minidle", default_min_idle() as i64)?
            .set_default("security.bcryptcost", default_bcrypt_cost() as i64)?
            // .set_default("security.jwtsecret", "defaultjwtsecret")?
            // Optional TOML file (don't error if missing)
            .add_source(File::with_name("poker-tracker").required(false))
            // Environment variables override (hierarchical keys use _ as separator)
            .add_source(Environment::default().separator("_"))
            .build()?;

        config.try_deserialize()
    }
}
