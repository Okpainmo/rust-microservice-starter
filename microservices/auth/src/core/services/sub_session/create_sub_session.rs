//! # Sub-Session Audit Service
//!
//! This module handles the creation of granular audit logs (sub-sessions).
//!
//! ### Automatic Metadata Extraction
//! To avoid manual boilerplate in controllers, this service uses a `tokio::task_local`
//! stored context (populated by `logging_middleware`) to automatically extract
//! the client's **IP Address** and **User-Agent**.
//!
//! If the service is called outside of a request context where metadata is not set,
//! it falls back to the `Option` fields provided in the [`CreateSubSession`] struct.

use crate::core::structs::sub_session::SubSession;
use axum::http::HeaderMap;
use sqlx::PgPool;
use tokio::task_local;
use uuid::Uuid;

task_local! {
    /// Task-local storage for request metadata: (Headers, Remote IP).
    pub static REQUEST_CONTEXT: (HeaderMap, Option<String>);
}

pub struct CreateSubSession {
    pub session_id: Uuid,
    pub user_id: i64,
    pub activity_type: String,
    pub activity_description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: String,
    pub request_path: String,
}

// ============================================================================
// Internal Extraction Helpers
// ============================================================================

/// Extracts the client IP address from proxy headers or connection info.
fn resolve_ip(headers: &HeaderMap, remote_ip: Option<String>) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or(remote_ip) // Use connection IP if headers are missing
}

/// Extracts the `User-Agent` string from headers.
fn resolve_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

// ============================================================================
// Service Function
// ============================================================================

pub async fn create_sub_session(
    db: &PgPool,
    sub_session: CreateSubSession,
) -> Result<SubSession, sqlx::Error> {
    // Attempt to resolve IP/UA from task-local REQUEST_CONTEXT
    let (header_ip, header_ua) = REQUEST_CONTEXT
        .try_with(|(headers, remote_ip)| {
            (
                resolve_ip(headers, remote_ip.clone()),
                resolve_user_agent(headers),
            )
        })
        .unwrap_or((None, None));

    let ip_address = header_ip.or(sub_session.ip_address);
    let user_agent = header_ua.or(sub_session.user_agent);

    sqlx::query_as::<_, SubSession>(
        r#"
        INSERT INTO sub_sessions (
            id,
            session_id,
            user_id,
            activity_type,
            activity_description,
            ip_address,
            user_agent,
            request_method,
            request_path
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id,
            creation_order,
            session_id,
            user_id,
            activity_type,
            activity_description,
            ip_address,
            user_agent,
            request_method,
            request_path,
            created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sub_session.session_id)
    .bind(sub_session.user_id)
    .bind(sub_session.activity_type)
    .bind(sub_session.activity_description)
    .bind(ip_address)
    .bind(user_agent)
    .bind(sub_session.request_method)
    .bind(sub_session.request_path)
    .fetch_one(db)
    .await
}
