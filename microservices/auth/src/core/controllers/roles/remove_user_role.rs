use crate::AppState;
use crate::core::services::role::user_role::remove_user_role;
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::middlewares::sessions_middleware::SessionsMiddlewareOutput;
use axum::extract::{Extension, State};
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RemoveUserRoleRequest {
    user_id: i64,
    role_id: String,
}

#[derive(Debug, Serialize)]
pub struct RemoveUserRoleResponseCore {
    data: (),
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RemoveUserRoleResponse {
    response_message: String,
    response: Option<RemoveUserRoleResponseCore>,
    error: Option<String>,
}

pub async fn remove_user_role_controller(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    method: Method,
    uri: Uri,
    Json(payload): Json<RemoveUserRoleRequest>,
) -> impl IntoResponse {
    let role_id = match Uuid::parse_str(&payload.role_id) {
        Ok(role_id) => role_id,
        Err(e) => {
            error!("INVALID ROLE ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(RemoveUserRoleResponse {
                    response_message: "Failed to remove user role".to_string(),
                    response: None,
                    error: Some("Invalid role id".to_string()),
                }),
            );
        }
    };

    match remove_user_role(&state.db, payload.user_id, role_id).await {
        Ok(_) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "remove_user_role".to_string(),
                    activity_description: Some("Remove user role endpoint accessed".to_string()),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE REMOVE USER ROLE SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RemoveUserRoleResponse {
                        response_message: "Failed to remove user role".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(RemoveUserRoleResponse {
                    response_message: "User role removed successfully".to_string(),
                    response: Some(RemoveUserRoleResponseCore {
                        data: (),
                        session_id: session_output.session.id.to_string(),
                        access_token: session_output.access_token,
                        refresh_token: session_output.session_token,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("FAILED TO REMOVE USER ROLE: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RemoveUserRoleResponse {
                    response_message: "Failed to remove user role".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
