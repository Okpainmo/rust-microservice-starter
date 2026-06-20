use crate::core::services::sub_session::create_sub_session::REQUEST_CONTEXT;
use axum::extract::ConnectInfo;
use axum::{extract::Request, middleware::Next, response::Response};
use std::net::SocketAddr;
use std::time::Instant;
use tracing::info;

// ============================================================================
// Logging Middleware
// ============================================================================

pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let start_time = Instant::now();
    let start_timestamp = chrono::Local::now();

    // 1. Capture Headers
    let headers = req.headers().clone();

    // 2. Capture Peer IP (if available)
    let remote_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip().to_string());

    // 3. Process the request within the metadata scope
    let response = REQUEST_CONTEXT
        .scope((headers, remote_ip), async move { next.run(req).await })
        .await;

    // Calculate duration and end time
    let duration = start_time.elapsed();
    let end_timestamp = chrono::Local::now();

    info!(
        "Path: {} | Start: {} | End: {} | Duration: {:.3}ms",
        path,
        start_timestamp.format("%H:%M:%S%.3f"),
        end_timestamp.format("%H:%M:%S%.3f"),
        duration.as_secs_f64() * 1000.0,
    );

    response
}
