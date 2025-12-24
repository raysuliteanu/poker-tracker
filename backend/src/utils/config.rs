use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PokerTrackerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub db_url: String, // Required, no default
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,
    #[serde(default = "default_db_min_idle")]
    pub db_min_idle: u32,
    pub jwt_secret: String, // Required, no default
    #[serde(default = "default_bcrypt_cost")]
    pub bcrypt_cost: u32,
}

// Default value functions
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_db_max_connections() -> u32 {
    100
}

fn default_db_min_idle() -> u32 {
    10
}

fn default_bcrypt_cost() -> u32 {
    bcrypt::DEFAULT_COST
}

impl PokerTrackerConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // Start with defaults
            .set_default("host", default_host())?
            .set_default("port", default_port() as i64)?
            .set_default("db_max_connections", default_db_max_connections() as i64)?
            .set_default("db_min_idle", default_db_min_idle() as i64)?
            .set_default("bcrypt_cost", default_bcrypt_cost() as i64)?
            // Optional TOML file (don't error if missing)
            .add_source(File::with_name("poker-tracker").required(false))
            // Environment variables override
            .add_source(Environment::default())
            .build()?;

        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_parsing_with_upper_case() {
        // Set environment variables in UPPER_CASE
        // Clean up any existing variables for test isolation
        unsafe {
            std::env::remove_var("DB_URL");
            std::env::remove_var("db_url");
            std::env::remove_var("JWT_SECRET");
            std::env::remove_var("jwt_secret");
            std::env::remove_var("BCRYPT_COST");
        }

        unsafe {
            std::env::set_var("DB_URL", "postgres://test:test@localhost/test");
            std::env::set_var("JWT_SECRET", "test-secret-key");
            std::env::set_var("BCRYPT_COST", "4");
            std::env::set_var("HOST", "0.0.0.0");
            std::env::set_var("PORT", "9090");
        }

        // Try to load config
        let result = PokerTrackerConfig::load();

        // Clean up environment variables
        unsafe {
            std::env::remove_var("DB_URL");
            std::env::remove_var("JWT_SECRET");
            std::env::remove_var("BCRYPT_COST");
            std::env::remove_var("HOST");
            std::env::remove_var("PORT");
        }

        // Assert that config loaded successfully
        assert!(
            result.is_ok(),
            "Config should load successfully, but got error: {:?}",
            result.err()
        );

        let config = result.unwrap();

        // Verify values were parsed correctly
        assert_eq!(
            config.db_url, "postgres://test:test@localhost/test",
            "db_url should match DB_URL env var"
        );
        assert_eq!(
            config.jwt_secret, "test-secret-key",
            "jwt_secret should match JWT_SECRET env var"
        );
        assert_eq!(
            config.bcrypt_cost, 4,
            "bcrypt_cost should match BCRYPT_COST env var"
        );
        assert_eq!(
            config.host, "0.0.0.0",
            "host should match HOST env var"
        );
        assert_eq!(config.port, 9090, "port should match PORT env var");
    }

    #[test]
    fn test_env_var_parsing_without_case_conversion() {
        // This test checks what happens if we use lowercase env vars
        // to understand the baseline behavior

        // Clean up for test isolation
        unsafe {
            std::env::remove_var("DB_URL");
            std::env::remove_var("db_url");
            std::env::remove_var("JWT_SECRET");
            std::env::remove_var("jwt_secret");
            std::env::remove_var("BCRYPT_COST");
        }

        unsafe {
            std::env::set_var("db_url", "postgres://test2:test2@localhost/test2");
            std::env::set_var("jwt_secret", "test-secret-key-2");
        }

        let config_result = Config::builder()
            .set_default("host", default_host())
            .unwrap()
            .set_default("port", default_port() as i64)
            .unwrap()
            .set_default("db_max_connections", default_db_max_connections() as i64)
            .unwrap()
            .set_default("db_min_idle", default_db_min_idle() as i64)
            .unwrap()
            .set_default("bcrypt_cost", default_bcrypt_cost() as i64)
            .unwrap()
            .add_source(File::with_name("poker-tracker").required(false))
            .add_source(Environment::default())
            .build()
            .unwrap()
            .try_deserialize::<PokerTrackerConfig>();

        unsafe {
            std::env::remove_var("db_url");
            std::env::remove_var("jwt_secret");
        }

        assert!(
            config_result.is_ok(),
            "Config with lowercase env vars should work, error: {:?}",
            config_result.err()
        );

        let config = config_result.unwrap();
        assert_eq!(config.db_url, "postgres://test2:test2@localhost/test2");
        assert_eq!(config.jwt_secret, "test-secret-key-2");
    }

    #[test]
    fn test_missing_required_fields() {
        // Ensure required fields cause an error when missing
        unsafe {
            std::env::remove_var("DB_URL");
            std::env::remove_var("db_url");
            std::env::remove_var("JWT_SECRET");
            std::env::remove_var("jwt_secret");
            std::env::remove_var("BCRYPT_COST");
        }

        let result = PokerTrackerConfig::load();

        assert!(
            result.is_err(),
            "Config should fail when required fields are missing"
        );
    }
}
