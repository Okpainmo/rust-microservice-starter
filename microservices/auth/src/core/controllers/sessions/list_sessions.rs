use crate::AppState;
use crate::core::controllers::sessions::lib::session_response::{
    SessionWithSubSessions, SessionsResponse, SessionsResponseCore,
};
use crate::core::services::session::find_sessions::find_all_sessions;
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::services::sub_session::find_sub_session::find_sub_sessions_by_session_id;
use crate::middlewares::sessions_middleware::SessionsMiddlewareOutput;
use axum::extract::{Extension, State};
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use tracing::error;

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    method: Method,
    uri: Uri,
) -> impl IntoResponse {
    let sessions = match find_all_sessions(&state.db).await {
        Ok(sessions) => sessions,
        Err(e) => {
            error!("FAILED TO LIST SESSIONS: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SessionsResponse::failure(
                    "Failed to list sessions",
                    e.to_string(),
                )),
            );
        }
    };

    let mut sessions_with_sub_sessions = Vec::with_capacity(sessions.len());

    for session in sessions {
        let sub_sessions = match find_sub_sessions_by_session_id(&state.db, session.id).await {
            Ok(sub_sessions) => sub_sessions,
            Err(e) => {
                error!("FAILED TO LIST SUB-SESSIONS: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SessionsResponse::failure(
                        "Failed to list sessions",
                        e.to_string(),
                    )),
                );
            }
        };

        sessions_with_sub_sessions.push(SessionWithSubSessions::new(session, sub_sessions));
    }

    if let Err(e) = create_sub_session(
        &state.db,
        CreateSubSession {
            session_id: session_output.session.id,
            user_id: session_output.user.id,
            activity_type: "list_sessions".to_string(),
            activity_description: Some("Accessed list-sessions end-point".to_string()),
            ip_address: None,
            user_agent: None,
            request_method: method.as_str().to_string(),
            request_path: uri.path().to_string(),
        },
    )
    .await
    {
        error!("FAILED TO CREATE LIST SESSIONS SUB-SESSION: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionsResponse::failure(
                "Failed to list sessions",
                e.to_string(),
            )),
        );
    }

    (
        StatusCode::OK,
        Json(SessionsResponse::success(
            "Sessions fetched successfully",
            SessionsResponseCore::new(
                sessions_with_sub_sessions,
                session_output.session.id.to_string(),
                session_output.access_token,
                session_output.session_token,
            ),
        )),
    )
}
