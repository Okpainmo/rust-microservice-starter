use axum::{Json, extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::Serialize;
use std::time::{Duration, Instant};
use tokio::time::timeout;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TimeoutErrorResponse {
    pub error: String,
    pub response_message: String,
}

// ============================================================================
// Timeout Middleware
// ============================================================================

use crate::AppState;
use axum::extract::State;

pub async fn timeout_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<TimeoutErrorResponse>)> {
    let path = req.uri().path().to_string();
    let start_time = Instant::now();
    let start_timestamp = chrono::Local::now();

    let timeout_secs = state
        .config
        .server
        .as_ref()
        .map(|s| s.request_timeout_secs)
        .unwrap_or(60);

    let timeout_duration = Duration::from_secs(timeout_secs);

    // Run the request inside a timeout future
    match timeout(timeout_duration, next.run(req)).await {
        Ok(response) => {
            let end_timestamp = chrono::Local::now();
            let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

            tracing::debug!(
                "[TIMEOUT MIDDLEWARE] Path: {} | Start: {} | End: {} | Duration: {:.3}ms",
                path,
                start_timestamp.format("%H:%M:%S%.3f"),
                end_timestamp.format("%H:%M:%S%.3f"),
                duration_ms,
            );

            Ok(response)
        }
        Err(_) => {
            let end_timestamp = chrono::Local::now();
            let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

            tracing::error!(
                "[TIMEOUT MIDDLEWARE] TIMEOUT! Path: {} | Start: {} | End: {} | Duration: {:.3}ms",
                path,
                start_timestamp.format("%H:%M:%S%.3f"),
                end_timestamp.format("%H:%M:%S%.3f"),
                duration_ms,
            );

            Err((
                StatusCode::REQUEST_TIMEOUT,
                Json(TimeoutErrorResponse {
                    error: "Request Timeout".to_string(),
                    response_message: format!(
                        "Request exceeded the maximum allowed time of {} seconds",
                        timeout_secs
                    ),
                }),
            ))
        }
    }
}
