use crate::AppState;
use crate::core::controllers::sessions::lib::session_response::{
    SessionDataResponseCore, SessionResponse, SessionWithSubSessions,
};
use crate::core::services::session::find_sessions::find_session_by_id;
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
use tracing::error;
use uuid::Uuid;

pub async fn get_session(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    Path(session_id): Path<String>,
    method: Method,
    uri: Uri,
) -> impl IntoResponse {
    let session_id = match Uuid::parse_str(&session_id) {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("INVALID SESSION ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(SessionResponse::failure(
                    "Failed to fetch session",
                    "Invalid session id",
                )),
            );
        }
    };

    let session = match find_session_by_id(&state.db, session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SessionResponse::failure(
                    "Failed to fetch session",
                    "Session not found",
                )),
            );
        }
        Err(e) => {
            error!("FAILED TO FETCH SESSION: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SessionResponse::failure(
                    "Failed to fetch session",
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
                    "Failed to fetch session",
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
            activity_type: "get_session".to_string(),
            activity_description: Some("Accessed get-session end-point".to_string()),
            ip_address: None,
            user_agent: None,
            request_method: method.as_str().to_string(),
            request_path: uri.path().to_string(),
        },
    )
    .await
    {
        error!("FAILED TO CREATE GET SESSION SUB-SESSION: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionResponse::failure(
                "Failed to fetch session",
                e.to_string(),
            )),
        );
    }

    (
        StatusCode::OK,
        Json(SessionResponse::success(
            "Session fetched successfully",
            SessionDataResponseCore::new(
                SessionWithSubSessions::new(session, sub_sessions),
                session_output.session.id.to_string(),
                session_output.access_token,
                session_output.session_token,
            ),
        )),
    )
}
