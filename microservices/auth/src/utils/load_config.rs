//! # Configuration Management
//!
//! This module handles loading and validating the application configuration from
//! multiple sources: base TOML files, environment-specific overrides, local
//! overrides, and environment variables.

use anyhow::{Context, Result};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::fmt;

/// Application-specific metadata section.
#[derive(Debug, Deserialize)]
pub struct AppSection {
    /// The name of the application.
    pub name: String,
    /// The current environment (e.g., development, production).
    pub environment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClientIntegrationsSection {
    #[serde(default)]
    pub allow_access_middleware: bool,

    #[serde(default)]
    pub allow_sessions_middleware: bool,

    #[serde(default)]
    pub allow_logging_middleware: bool,

    #[serde(default)]
    pub allow_request_timeout_middleware: bool,

    #[serde(default)]
    pub allow_rate_limit_middleware: bool,

    #[serde(default)]
    pub allow_admin_routes_protector_middleware: bool,
}

// #[derive(Debug, Deserialize)]
// pub struct ObservabilitySection {
//     pub enable_tracing: bool,
//     pub enable_metrics: bool,
// }

#[derive(Debug, Deserialize)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSection {
    pub engine: String,
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
    pub name: String,
    pub max_connections: u32,
    pub connect_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct AuthSection {
    pub jwt_secret: String,
    pub jwt_access_expiration_time_in_hours: u64,
    pub jwt_refresh_expiration_time_in_hours: u64,
    pub jwt_one_time_password_lifetime_in_minutes: u64,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitSection {
    pub enabled: bool,
    pub requests_per_window: u64,
    pub window_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct MeshSection {
    #[serde(default)]
    pub enabled: bool,
    pub url: String,
    pub token: Option<String>,
    pub service_name: String,
    pub service_version: String,
    pub advertise_host: Option<String>,
    pub heartbeat_interval_secs: u64,
    pub external_host: Option<String>,
    pub external_port: Option<u16>,
    pub external_scheme: String,
}

/// Root configuration structure containing all application settings.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub client_integrations: ClientIntegrationsSection,
    // pub observability: ObservabilitySection,

    // Optional / currently commented-out sections
    pub server: Option<ServerSection>,
    pub database: Option<DatabaseSection>,
    pub auth: Option<AuthSection>,
    pub rate_limit: Option<RateLimitSection>,
    pub mesh: Option<MeshSection>,
}

/// Loads the application configuration.
///
/// Order of precedence (highest to lowest):
/// 1. Environment variables (prefixed with `APP__`) - overrides every other configuration setup
/// 2. `config/local.toml` - overrides `config/{APP__ENV}.toml` and `config/base.toml`
/// 3. `config/{APP__ENV}.toml` - overrides `config/base.toml`
/// 4. `config/base.toml` - default values
pub fn load_config() -> Result<AppConfig> {
    // Determine environment
    let env = std::env::var("APP__ENV").context("APP__ENV environment variable is not set! Please set it to 'development', 'production', etc.")?;

    // Build configuration
    let builder = Config::builder()
        // Base config is required
        .add_source(File::with_name("config/base").required(true))
        // Environment-specific overrides (optional)
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        // Local overrides (optional, for dev machines)
        .add_source(File::with_name("config/local").required(false))
        // Environment variable overrides
        .add_source(
            Environment::default()
                .separator("__") // maps APP__SECTION__FIELD → section.field
                .prefix("APP") // all vars must start with APP__
                .try_parsing(true), // parse numbers/booleans automatically
        );

    /**************** EXPLAINING THE MAPPING RULE FOR THE [ABOVE] FINAL ENV OVERRIDES ****************
    # Mapping Rule (exact)

    APP__<SECTION>__<FIELD>=value - E.g. APP__SERVER__PORT=9000

    Lowercase / uppercase differences are normalized(handled without manual intervention).

    So this TOML:

    [server]
    port = 8080

    will be overridden by:

    APP__SERVER__PORT=9000

    If the names don’t align, nothing happens.

    Example (❌ no override):

    SERVER_PORT=9000

    This does nothing unless you explicitly read it in code.

    **************** EXPLAINING THE MAPPING RULE FOR THE [ABOVE] FINAL ENV OVERRIDES ****************/

    builder
        .build()
        .context("Failed to build config")?
        .try_deserialize()
        .context("Invalid config shape")
}

#[derive(Debug)]
pub enum ConfigError {
    MissingAppName,
    InvalidServerPort,
    MissingServerSection,
    MissingDatabaseSection,
    MissingDatabaseName,
    MissingDatabaseUser,
    MissingDatabasePassword,
    MissingAuthSection,
    MissingJwtSecret,
    MissingMeshUrl,
    MissingMeshServiceName,
    MissingMeshServiceVersion,
    InvalidMeshHeartbeatInterval,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingAppName => write!(f, "app.name cannot be empty"),
            ConfigError::InvalidServerPort => write!(f, "server.port cannot be 0"),
            ConfigError::MissingServerSection => write!(f, "server section is missing"),
            ConfigError::MissingDatabaseSection => write!(f, "database section is missing"),
            ConfigError::MissingDatabaseName => write!(f, "database.name cannot be empty"),
            ConfigError::MissingDatabaseUser => write!(f, "database.user cannot be empty"),
            ConfigError::MissingDatabasePassword => write!(f, "database.password cannot be empty"),
            ConfigError::MissingAuthSection => write!(f, "auth section is missing"),
            ConfigError::MissingJwtSecret => write!(f, "auth.jwt_secret cannot be empty"),
            ConfigError::MissingMeshUrl => write!(f, "mesh.url cannot be empty"),
            ConfigError::MissingMeshServiceName => write!(f, "mesh.service_name cannot be empty"),
            ConfigError::MissingMeshServiceVersion => {
                write!(f, "mesh.service_version cannot be empty")
            }
            ConfigError::InvalidMeshHeartbeatInterval => {
                write!(f, "mesh.heartbeat_interval_secs cannot be 0")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    pub fn validate(&self) -> std::result::Result<(), ConfigError> {
        // Check app name
        if self.app.name.trim().is_empty() {
            return Err(ConfigError::MissingAppName);
        }

        // Check server
        let server = self
            .server
            .as_ref()
            .ok_or(ConfigError::MissingServerSection)?;
        if server.port == 0 {
            return Err(ConfigError::InvalidServerPort);
        }

        // Check database
        let database = self
            .database
            .as_ref()
            .ok_or(ConfigError::MissingDatabaseSection)?;
        if database.name.trim().is_empty() {
            return Err(ConfigError::MissingDatabaseName);
        }
        if database
            .user
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(ConfigError::MissingDatabaseUser);
        }
        if database
            .password
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(ConfigError::MissingDatabasePassword);
        }

        // Check auth
        let auth = self.auth.as_ref().ok_or(ConfigError::MissingAuthSection)?;
        if auth.jwt_secret.trim().is_empty() {
            return Err(ConfigError::MissingJwtSecret);
        }

        if let Some(mesh) = self.mesh.as_ref().filter(|mesh| mesh.enabled) {
            if mesh.url.trim().is_empty() {
                return Err(ConfigError::MissingMeshUrl);
            }
            if mesh.service_name.trim().is_empty() {
                return Err(ConfigError::MissingMeshServiceName);
            }
            if mesh.service_version.trim().is_empty() {
                return Err(ConfigError::MissingMeshServiceVersion);
            }
            if mesh.heartbeat_interval_secs == 0 {
                return Err(ConfigError::InvalidMeshHeartbeatInterval);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_app_section() -> AppSection {
        AppSection {
            name: "Test App".to_string(),
            environment: Some("development".to_string()),
        }
    }

    fn valid_auth_section() -> AuthSection {
        AuthSection {
            jwt_secret: "secret".to_string(),
            jwt_access_expiration_time_in_hours: 1,
            jwt_refresh_expiration_time_in_hours: 24,
            jwt_one_time_password_lifetime_in_minutes: 5,
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig {
            app: valid_app_section(),
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: true,
                allow_sessions_middleware: true,
                allow_logging_middleware: true,
                allow_request_timeout_middleware: true,
                allow_rate_limit_middleware: false,
                allow_admin_routes_protector_middleware: true,
            },
            // observability: ObservabilitySection {
            //     enable_tracing: true,
            //     enable_metrics: true,
            // },
            server: Some(ServerSection {
                host: "127.0.0.1".to_string(),
                port: 8080,
                request_timeout_secs: 60,
            }),
            database: Some(DatabaseSection {
                engine: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: Some("test".to_string()),
                password: Some("pass".to_string()),
                name: "db".to_string(),
                max_connections: 5,
                connect_timeout_secs: 3,
            }),
            auth: Some(valid_auth_section()),
            rate_limit: None,
            mesh: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_app_name() {
        let mut config = AppConfig {
            app: valid_app_section(),
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: false,
                allow_sessions_middleware: false,
                allow_logging_middleware: false,
                allow_request_timeout_middleware: false,
                allow_rate_limit_middleware: false,
                allow_admin_routes_protector_middleware: false,
            },
            // observability: ObservabilitySection {
            //     enable_tracing: false,
            //     enable_metrics: false,
            // },
            server: Some(ServerSection {
                host: "127.0.0.1".to_string(),
                port: 8080,
                request_timeout_secs: 60,
            }),
            database: Some(DatabaseSection {
                engine: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: Some("test".to_string()),
                password: Some("pass".to_string()),
                name: "db".to_string(),
                max_connections: 5,
                connect_timeout_secs: 3,
            }),
            auth: Some(valid_auth_section()),
            rate_limit: None,
            mesh: None,
        };
        config.app.name = "".to_string();

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "app.name cannot be empty");
    }

    #[test]
    fn test_validate_invalid_port() {
        let config = AppConfig {
            app: valid_app_section(),
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: false,
                allow_sessions_middleware: false,
                allow_logging_middleware: false,
                allow_request_timeout_middleware: false,
                allow_rate_limit_middleware: false,
                allow_admin_routes_protector_middleware: false,
            },
            // observability: ObservabilitySection {
            //     enable_tracing: false,
            //     enable_metrics: false,
            // },
            server: Some(ServerSection {
                host: "127.0.0.1".to_string(),
                port: 0,
                request_timeout_secs: 60,
            }),
            database: Some(DatabaseSection {
                engine: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: Some("test".to_string()),
                password: Some("pass".to_string()),
                name: "db".to_string(),
                max_connections: 5,
                connect_timeout_secs: 3,
            }),
            auth: Some(valid_auth_section()),
            rate_limit: None,
            mesh: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "server.port cannot be 0");
    }

    #[test]
    fn test_validate_missing_database_fields() {
        let config = AppConfig {
            app: valid_app_section(),
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: false,
                allow_sessions_middleware: false,
                allow_logging_middleware: false,
                allow_request_timeout_middleware: false,
                allow_rate_limit_middleware: false,
                allow_admin_routes_protector_middleware: false,
            },
            // observability: ObservabilitySection {
            //     enable_tracing: false,
            //     enable_metrics: false,
            // },
            server: Some(ServerSection {
                host: "127.0.0.1".to_string(),
                port: 8080,
                request_timeout_secs: 60,
            }),
            database: Some(DatabaseSection {
                engine: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: None,
                password: Some("pass".to_string()),
                name: "db".to_string(),
                max_connections: 5,
                connect_timeout_secs: 3,
            }),
            auth: Some(valid_auth_section()),
            rate_limit: None,
            mesh: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "database.user cannot be empty"
        );
    }

    #[test]
    fn test_validate_missing_server_section() {
        let config = AppConfig {
            app: valid_app_section(),
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: false,
                allow_sessions_middleware: false,
                allow_logging_middleware: false,
                allow_request_timeout_middleware: false,
                allow_rate_limit_middleware: false,
                allow_admin_routes_protector_middleware: false,
            },
            // observability: ObservabilitySection {
            //     enable_tracing: false,
            //     enable_metrics: false,
            // },
            server: None,
            database: None,
            auth: None,
            rate_limit: None,
            mesh: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "server section is missing");
    }

    #[test]
    fn test_validate_rejects_enabled_mesh_with_missing_url() {
        let config = AppConfig {
            app: valid_app_section(),
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: false,
                allow_sessions_middleware: false,
                allow_logging_middleware: false,
                allow_request_timeout_middleware: false,
                allow_rate_limit_middleware: false,
                allow_admin_routes_protector_middleware: false,
            },
            server: Some(ServerSection {
                host: "127.0.0.1".to_string(),
                port: 8080,
                request_timeout_secs: 60,
            }),
            database: Some(DatabaseSection {
                engine: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: Some("test".to_string()),
                password: Some("pass".to_string()),
                name: "db".to_string(),
                max_connections: 5,
                connect_timeout_secs: 3,
            }),
            auth: Some(valid_auth_section()),
            rate_limit: None,
            mesh: Some(MeshSection {
                enabled: true,
                url: "".to_string(),
                token: None,
                service_name: "auth-service".to_string(),
                service_version: "1.0.0".to_string(),
                advertise_host: None,
                heartbeat_interval_secs: 5,
                external_host: None,
                external_port: None,
                external_scheme: "http".to_string(),
            }),
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "mesh.url cannot be empty");
    }
}
