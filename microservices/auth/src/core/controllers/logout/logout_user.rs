use crate::AppState;
use crate::core::services::session::revoke_session::revoke_session_by_id_and_user_id;
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::services::user::find_user::find_user_profile_by_email;
use crate::core::services::user::update_user::{UpdateUser, update_user_by_email};
use axum::extract::State;
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    response_message: String,
    response: Option<()>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    user_email: String,
    session_id: String,
}

pub async fn logout_user(
    State(state): State<AppState>,
    cookies: Cookies,
    method: Method,
    uri: Uri,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    // Remove auth cookie
    let mut cookie = Cookie::new("auth_cookie", "");
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
    cookies.remove(cookie);

    let user = match find_user_profile_by_email(&state.db, &payload.user_email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("USER LOGOUT WAS UNSUCCESSFUL: USER NOT FOUND!");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some("User not found".to_string()),
                    response: None,
                }),
            );
        }
        Err(e) => {
            error!("USER LOOKUP FAILED DURING LOGOUT: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some(e.to_string()),
                    response: None,
                }),
            );
        }
    };

    let session_id = match Uuid::parse_str(&payload.session_id) {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("INVALID SESSION ID DURING LOGOUT: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some("Invalid session id".to_string()),
                    response: None,
                }),
            );
        }
    };

    let revoked_session =
        match revoke_session_by_id_and_user_id(&state.db, session_id, user.id).await {
            Ok(Some(session)) => session,
            Ok(None) => {
                error!("USER LOGOUT WAS UNSUCCESSFUL: SESSION NOT FOUND!");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LogoutResponse {
                        response_message: "Logout failed".to_string(),
                        error: Some("Session not found".to_string()),
                        response: None,
                    }),
                );
            }
            Err(e) => {
                error!("SESSION REVOCATION FAILED DURING LOGOUT: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LogoutResponse {
                        response_message: "Logout failed".to_string(),
                        error: Some(e.to_string()),
                        response: None,
                    }),
                );
            }
        };

    if let Err(e) = create_sub_session(
        &state.db,
        CreateSubSession {
            session_id: revoked_session.id,
            user_id: user.id,
            activity_type: "logout".to_string(),
            activity_description: Some("User logged out".to_string()),
            ip_address: None,
            user_agent: None,
            request_method: method.as_str().to_string(),
            request_path: uri.path().to_string(),
        },
    )
    .await
    {
        error!("FAILED TO CREATE LOGOUT SUB-SESSION: {}", e);
    }

    let user = update_user_by_email(
        &state.db,
        &payload.user_email,
        UpdateUser {
            access_token: Some("".to_string()),
            refresh_token: Some("".to_string()),
            is_logged_out: Some(true),
        },
    )
    .await;

    match user {
        Ok(rows_affected) if rows_affected > 0 => (
            StatusCode::OK,
            Json(LogoutResponse {
                response_message: "Logout successful".to_string(),
                error: None,
                response: None,
            }),
        ),
        Ok(_) => {
            error!("USER LOGOUT WAS UNSUCCESSFUL!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some("User not found".to_string()),
                    response: None,
                }),
            )
        }
        Err(e) => {
            error!("USER LOGOUT WAS UNSUCCESSFUL!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some(e.to_string()),
                    response: None,
                }),
            )
        }
    }
}
