use crate::AppState;
use crate::core::controllers::sessions::lib::session_response::{
    SessionDataResponseCore, SessionResponse, SessionWithSubSessions,
};
use crate::core::controllers::sessions::lib::update_session_request::UpdateSessionRequest;
use crate::core::services::session::update_session::update_session_expires_at;
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::services::sub_session::find_sub_session::find_sub_sessions_by_session_id;
use crate::middlewares::sessions_middleware::SessionsMiddlewareOutput;
use axum::extract::{Extension, Path, State};
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use tracing::error;
use uuid::Uuid;

pub async fn update_session(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    Path(session_id): Path<String>,
    method: Method,
    uri: Uri,
    Json(payload): Json<UpdateSessionRequest>,
) -> impl IntoResponse {
    let session_id = match Uuid::parse_str(&session_id) {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("INVALID SESSION ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(SessionResponse::failure(
                    "Failed to update session",
                    "Invalid session id",
                )),
            );
        }
    };

    let expires_at =
        match DateTime::<Utc>::from_timestamp_millis(payload.expires_at_in_milliseconds) {
            Some(expires_at) => expires_at.naive_utc(),
            None => {
                error!(
                    "INVALID SESSION EXPIRATION TIMESTAMP: {}",
                    payload.expires_at_in_milliseconds
                );
                return (
                    StatusCode::BAD_REQUEST,
                    Json(SessionResponse::failure(
                        "Failed to update session",
                        "Invalid expiration timestamp",
                    )),
                );
            }
        };

    let session = match update_session_expires_at(&state.db, session_id, expires_at).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SessionResponse::failure(
                    "Failed to update session",
                    "Session not found",
                )),
            );
        }
        Err(e) => {
            error!("FAILED TO UPDATE SESSION: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SessionResponse::failure(
                    "Failed to update session",
                    e.to_string(),
                )),
            );
        }
    };

    let sub_sessions = match find_sub_sessions_by_session_id(&state.db, session.id).await {
        Ok(sub_sessions) => sub_sessions,
        Err(e) => {
            error!("FAILED TO FETCH SUB-SESSIONS: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SessionResponse::failure(
                    "Failed to update session",
                    e.to_string(),
                )),
            );
        }
    };

    if let Err(e) = create_sub_session(
        &state.db,
        CreateSubSession {
            session_id: session_output.session.id,
            user_id: session_output.user.id,
            activity_type: "update_session".to_string(),
            activity_description: Some("Accessed update-session end-point".to_string()),
            ip_address: None,
            user_agent: None,
            request_method: method.as_str().to_string(),
            request_path: uri.path().to_string(),
        },
    )
    .await
    {
        error!("FAILED TO CREATE UPDATE SESSION SUB-SESSION: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionResponse::failure(
                "Failed to update session",
                e.to_string(),
            )),
        );
    }

    (
        StatusCode::OK,
        Json(SessionResponse::success(
            "Session updated successfully",
            SessionDataResponseCore::new(
                SessionWithSubSessions::new(session, sub_sessions),
                session_output.session.id.to_string(),
                session_output.access_token,
                session_output.session_token,
            ),
        )),
    )
}
