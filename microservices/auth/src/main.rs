//! # Chat Auth Server Binary
//!
//! The entry point for the authentication server. This binary handles:
//! - Environment variable loading.
//! - Logging initialization.
//! - Configuration validation.
//! - Database connection establishment.
//! - Server binding and execution.

use auth::db::connect_postgres::connect_pg;
use auth::middlewares::rate_limit_middleware::new_rate_limit_store;
use auth::utils::load_config::{AppConfig, load_config};
use auth::utils::load_env::load_env;
use auth::utils::registry_client::{MeshRegistryClient, MeshRegistryClientConfig};
use auth::{AppState, create_app};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info};
use tracing_subscriber::fmt::time::SystemTime;

/// Initializes the global tracing subscriber with JSON formatting.
fn initialize_logging() {
    tracing_subscriber::fmt()
        .json()
        .with_timer(SystemTime)
        .with_level(true)
        .init();
}

#[tokio::main]
async fn main() {
    load_env();
    initialize_logging();

    let app_config = load_config();

    // println!("{:?}", app_config);

    let clean_config = match app_config {
        Ok(config) => {
            if let Err(e) = config.validate() {
                let error = format!(
                    "SERVER START-UP ERROR: FAILED TO LOAD SERVER CONFIGURATIONS, {}",
                    e
                );
                error!("{}", error);
                std::process::exit(1);
            }

            config
        }
        Err(e) => {
            let error = format!(
                "SERVER START-UP ERROR: FAILED TO LOAD SERVER CONFIGURATIONS, {}",
                e
            );
            error!("{}", error);
            std::process::exit(1);
        }
    };

    let db_config = match clean_config.database.as_ref() {
        Some(config) => config,
        None => {
            error!("SERVER START-UP ERROR: DATABASE CONFIGURATION IS MISSING!");
            std::process::exit(1);
        }
    };

    let db_user = match db_config.user.as_deref() {
        Some(user) => user,
        None => {
            error!("SERVER START-UP ERROR: DATABASE USER IS MISSING!");
            std::process::exit(1);
        }
    };

    let db_password = match db_config.password.as_deref() {
        Some(password) => password,
        None => {
            error!("SERVER START-UP ERROR: DATABASE PASSWORD IS MISSING!");
            std::process::exit(1);
        }
    };

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_config.host, db_config.port, db_config.name
    );

    let db_pool = connect_pg(
        database_url.clone(),
        db_config.max_connections,
        db_config.connect_timeout_secs,
    )
    .await;

    let state = AppState {
        config: Arc::new(clean_config),
        db: db_pool,
        rate_limit_store: new_rate_limit_store(),
    };

    let app = create_app(state.clone());

    let host = state
        .config
        .server
        .as_ref()
        .map(|s| s.host.as_str())
        .unwrap_or("127.0.0.1");
    let port = state.config.server.as_ref().map(|s| s.port).unwrap_or(8000);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid server address");

    let db_config_ref = state.config.database.as_ref().unwrap();

    let slice_db_url = format!(
        "postgres://...@{}:{}/..",
        db_config_ref.host, db_config_ref.port,
    );

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            print!(
                "
                .................................................
                Connected to DB: {}
                Environment: {}
                Status: DB connected successfully
                .................................................

                Server running on http://{}
                ",
                slice_db_url,
                state.config.app.environment.as_deref().unwrap_or("unknown"),
                addr
            );
            listener
        }
        Err(e) => {
            error!("SERVER INITIALIZATION ERROR: {}!", e);
            std::process::exit(1);
        }
    };

    let bound_port = listener
        .local_addr()
        .map(|addr| addr.port())
        .unwrap_or(port);
    let mesh_registration = register_with_mesh_until_ready(&state.config, host, bound_port).await;

    let server_result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;

    if let Some((registry, heartbeat)) = mesh_registration {
        heartbeat.abort();
        if let Err(e) = registry.unregister().await {
            error!("MESH UNREGISTRATION ERROR: {}!", e);
        }
    }

    match server_result {
        Ok(_) => {
            info!("Graceful server shutdown!");
        }
        Err(e) => {
            error!("SERVER SHUTDOWN ERROR: {}!", e);
        }
    }
}

async fn register_with_mesh_until_ready(
    config: &AppConfig,
    server_host: &str,
    service_port: u16,
) -> Option<(MeshRegistryClient, JoinHandle<()>)> {
    let mesh_config = config.mesh.as_ref().filter(|mesh| mesh.enabled)?;
    let advertised_host = mesh_config
        .advertise_host
        .as_deref()
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| server_host.to_string());

    let registry = MeshRegistryClient::new(MeshRegistryClientConfig {
        mesh_url: mesh_config.url.clone(),
        mesh_token: mesh_config.token.clone(),
        advertised_host,
        service_name: mesh_config.service_name.clone(),
        service_version: mesh_config.service_version.clone(),
        service_port,
        container_id: env::var("HOSTNAME").ok(),
        external_host: mesh_config.external_host.clone(),
        external_port: mesh_config.external_port,
        external_scheme: mesh_config.external_scheme.clone(),
    });

    loop {
        match registry.register().await {
            Ok(endpoint) => {
                info!(
                    service_name = %mesh_config.service_name,
                    service_version = %mesh_config.service_version,
                    url = %endpoint.url,
                    internal_ip = %endpoint.internal_ip,
                    internal_port = endpoint.internal_port,
                    "Service registered with mesh"
                );

                let heartbeat = registry.start_heartbeat(mesh_config.heartbeat_interval_secs);
                break Some((registry, heartbeat));
            }
            Err(e) => {
                error!(
                    service_name = %mesh_config.service_name,
                    service_version = %mesh_config.service_version,
                    error = %format!("{e:#}"),
                    "Mesh registration failed; retrying"
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
