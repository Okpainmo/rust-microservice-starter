use crate::AppState;
use crate::core::controllers::sessions::lib::session_response::{
    SessionWithSubSessions, SessionsResponse, SessionsResponseCore,
};
use crate::core::services::session::find_sessions::find_sessions_by_user_id;
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

pub async fn list_user_sessions(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    Path(user_id): Path<String>,
    method: Method,
    uri: Uri,
) -> impl IntoResponse {
    let user_id = match user_id.parse::<i64>() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("INVALID USER ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(SessionsResponse::failure(
                    "Failed to list user sessions",
                    "Invalid user id",
                )),
            );
        }
    };

    let sessions = match find_sessions_by_user_id(&state.db, user_id).await {
        Ok(sessions) => sessions,
        Err(e) => {
            error!("FAILED TO LIST USER SESSIONS: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SessionsResponse::failure(
                    "Failed to list user sessions",
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
                error!("FAILED TO LIST USER SUB-SESSIONS: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SessionsResponse::failure(
                        "Failed to list user sessions",
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
            activity_type: "list_user_sessions".to_string(),
            activity_description: Some("Accessed list-user-sessions end-point".to_string()),
            ip_address: None,
            user_agent: None,
            request_method: method.as_str().to_string(),
            request_path: uri.path().to_string(),
        },
    )
    .await
    {
        error!("FAILED TO CREATE LIST USER SESSIONS SUB-SESSION: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionsResponse::failure(
                "Failed to list user sessions",
                e.to_string(),
            )),
        );
    }

    (
        StatusCode::OK,
        Json(SessionsResponse::success(
            "User sessions fetched successfully",
            SessionsResponseCore::new(
                sessions_with_sub_sessions,
                session_output.session.id.to_string(),
                session_output.access_token,
                session_output.session_token,
            ),
        )),
    )
}
