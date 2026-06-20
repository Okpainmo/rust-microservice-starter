//! # Chat Auth Server Library
//!
//! This crate provides the core logic for the authentication server, including
//! router setup, state management, and middleware integration.

use crate::core::router::auth_routes;
use crate::middlewares::logging_middleware::logging_middleware;
use crate::middlewares::rate_limit_middleware::{RateLimitStore, rate_limit_middleware};
use crate::middlewares::request_timeout_middleware::timeout_middleware;
use crate::utils::load_config::AppConfig;
use axum::{Router, middleware};
use sqlx::PgPool;
use std::sync::Arc;

pub mod core;
pub mod db;
pub mod middlewares;
pub mod utils;

/// Global application state shared across all routes and middlewares.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Application configuration loaded from TOML and environment variables.
    pub config: Arc<AppConfig>,
    /// Thread-safe PostgreSQL connection pool.
    pub db: PgPool,
    /// In-memory fixed-window rate limit buckets.
    pub rate_limit_store: RateLimitStore,
}

/// Creates the main Axum application router.
///
/// This function:
/// - Nests the authentication routes under `/api/v1/auth`.
/// - Integrates logging and request timeout middlewares.
/// - Provides the global `AppState` to all handlers.
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth_routes(&state))
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            timeout_middleware,
        ))
        .with_state(state)
}
