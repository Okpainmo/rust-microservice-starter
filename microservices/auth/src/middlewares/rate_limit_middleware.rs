use crate::AppState;
use axum::{
    Json,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tracing::warn;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug)]
pub struct RateLimitBucket {
    pub request_count: u64,
    pub window_started_at: Instant,
}

#[derive(Debug, Serialize)]
pub struct RateLimitErrorResponse {
    pub error: String,
    pub response_message: String,
}

#[derive(Debug)]
struct RateLimitDecision {
    allowed: bool,
    limit: u64,
    remaining: u64,
    retry_after_secs: u64,
    reset_after_secs: u64,
}

pub type RateLimitStore = Arc<Mutex<HashMap<String, RateLimitBucket>>>;

pub fn new_rate_limit_store() -> RateLimitStore {
    Arc::new(Mutex::new(HashMap::new()))
}

// ============================================================================
// Internal Helpers
// ============================================================================

fn resolve_client_ip(req: &Request) -> String {
    req.headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(addr)| addr.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn rate_limit_key(req: &Request) -> String {
    resolve_client_ip(req)
}

fn evaluate_rate_limit(
    store: &RateLimitStore,
    key: String,
    requests_per_window: u64,
    window_duration: Duration,
) -> RateLimitDecision {
    let now = Instant::now();
    let mut buckets = store
        .lock()
        .expect("rate limit store mutex should not be poisoned");

    let bucket = buckets.entry(key).or_insert_with(|| RateLimitBucket {
        request_count: 0,
        window_started_at: now,
    });

    let elapsed = now.saturating_duration_since(bucket.window_started_at);

    if elapsed >= window_duration {
        bucket.request_count = 0;
        bucket.window_started_at = now;
    }

    let elapsed = now.saturating_duration_since(bucket.window_started_at);
    let reset_after_secs = window_duration.saturating_sub(elapsed).as_secs();

    if bucket.request_count >= requests_per_window {
        return RateLimitDecision {
            allowed: false,
            limit: requests_per_window,
            remaining: 0,
            retry_after_secs: reset_after_secs.max(1),
            reset_after_secs,
        };
    }

    bucket.request_count += 1;
    let remaining = requests_per_window.saturating_sub(bucket.request_count);

    RateLimitDecision {
        allowed: true,
        limit: requests_per_window,
        remaining,
        retry_after_secs: 0,
        reset_after_secs,
    }
}

fn unix_reset_timestamp(reset_after_secs: u64) -> String {
    let reset_at = SystemTime::now()
        .checked_add(Duration::from_secs(reset_after_secs))
        .unwrap_or_else(SystemTime::now);

    reset_at
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn add_rate_limit_headers(headers: &mut HeaderMap, decision: &RateLimitDecision) {
    if let Ok(value) = HeaderValue::from_str(&decision.limit.to_string()) {
        headers.insert("x-ratelimit-limit", value);
    }

    if let Ok(value) = HeaderValue::from_str(&decision.remaining.to_string()) {
        headers.insert("x-ratelimit-remaining", value);
    }

    if let Ok(value) = HeaderValue::from_str(&unix_reset_timestamp(decision.reset_after_secs)) {
        headers.insert("x-ratelimit-reset", value);
    }

    if !decision.allowed
        && let Ok(value) = HeaderValue::from_str(&decision.retry_after_secs.to_string())
    {
        headers.insert("retry-after", value);
    }
}

// ============================================================================
// Rate Limit Middleware
// ============================================================================

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let integrations = &state.config.client_integrations;
    let rate_limit_config = match state.config.rate_limit.as_ref() {
        Some(config) if integrations.allow_rate_limit_middleware && config.enabled => config,
        _ => return next.run(req).await,
    };

    let requests_per_window = rate_limit_config.requests_per_window.max(1);
    let window_duration = Duration::from_secs(rate_limit_config.window_secs.max(1));
    let key = rate_limit_key(&req);

    let decision = evaluate_rate_limit(
        &state.rate_limit_store,
        key.clone(),
        requests_per_window,
        window_duration,
    );

    if !decision.allowed {
        warn!("RATE LIMIT EXCEEDED: key={}", key);

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RateLimitErrorResponse {
                error: "Too Many Requests".to_string(),
                response_message: "Rate limit exceeded. Please try again later.".to_string(),
            }),
        )
            .into_response();

        add_rate_limit_headers(response.headers_mut(), &decision);

        return response;
    }

    let mut response = next.run(req).await;
    add_rate_limit_headers(response.headers_mut(), &decision);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_config::{
        AppConfig, AppSection, AuthSection, ClientIntegrationsSection, DatabaseSection,
        RateLimitSection, ServerSection,
    };
    use axum::{Router, routing::get};
    use axum_test::TestServer;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use std::time::Duration;

    fn test_config(
        allow_rate_limit_middleware: bool,
        requests_per_window: u64,
        window_secs: u64,
    ) -> AppConfig {
        AppConfig {
            app: AppSection {
                name: "test_app".to_string(),
                environment: Some("test".to_string()),
            },
            client_integrations: ClientIntegrationsSection {
                allow_access_middleware: false,
                allow_sessions_middleware: false,
                allow_logging_middleware: false,
                allow_request_timeout_middleware: false,
                allow_rate_limit_middleware,
                allow_admin_routes_protector_middleware: false,
            },
            server: Some(ServerSection {
                host: "127.0.0.1".to_string(),
                port: 8000,
                request_timeout_secs: 60,
            }),
            database: Some(DatabaseSection {
                engine: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: Some("test".to_string()),
                password: Some("test".to_string()),
                name: "test".to_string(),
                max_connections: 1,
                connect_timeout_secs: 1,
            }),
            auth: Some(AuthSection {
                jwt_secret: "test_secret".to_string(),
                jwt_access_expiration_time_in_hours: 1,
                jwt_refresh_expiration_time_in_hours: 24,
                jwt_one_time_password_lifetime_in_minutes: 5,
            }),
            rate_limit: Some(RateLimitSection {
                enabled: true,
                requests_per_window,
                window_secs,
            }),
            mesh: None,
        }
    }

    fn test_server(
        allow_rate_limit_middleware: bool,
        requests_per_window: u64,
        window_secs: u64,
    ) -> TestServer {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://test:test@localhost:5432/test")
            .expect("test database pool should be created lazily");

        let state = AppState {
            config: Arc::new(test_config(
                allow_rate_limit_middleware,
                requests_per_window,
                window_secs,
            )),
            db,
            rate_limit_store: new_rate_limit_store(),
        };

        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                rate_limit_middleware,
            ))
            .with_state(state);

        TestServer::new(app).expect("test server should be created")
    }

    #[tokio::test]
    async fn allows_requests_under_limit() {
        let server = test_server(true, 2, 60);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.10")
            .await
            .assert_status(StatusCode::OK);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.10")
            .await
            .assert_status(StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_requests_above_limit() {
        let server = test_server(true, 1, 60);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.20")
            .await
            .assert_status(StatusCode::OK);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.20")
            .await
            .assert_status(StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn tracks_different_ips_independently() {
        let server = test_server(true, 1, 60);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.30")
            .await
            .assert_status(StatusCode::OK);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.31")
            .await
            .assert_status(StatusCode::OK);
    }

    #[tokio::test]
    async fn skips_rate_limiting_when_feature_flag_is_disabled() {
        let server = test_server(false, 1, 60);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.40")
            .await
            .assert_status(StatusCode::OK);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.40")
            .await
            .assert_status(StatusCode::OK);
    }

    #[tokio::test]
    async fn allows_requests_after_window_resets() {
        let server = test_server(true, 1, 1);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.50")
            .await
            .assert_status(StatusCode::OK);

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.50")
            .await
            .assert_status(StatusCode::TOO_MANY_REQUESTS);

        tokio::time::sleep(Duration::from_millis(1_100)).await;

        server
            .get("/")
            .add_header("x-forwarded-for", "203.0.113.50")
            .await
            .assert_status(StatusCode::OK);
    }
}
