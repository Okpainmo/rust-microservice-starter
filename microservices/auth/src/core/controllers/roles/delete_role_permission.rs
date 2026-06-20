use crate::AppState;
use crate::core::services::role::role_permission::delete_role_permission;
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
pub struct DeleteRolePermissionRequest {
    role_id: String,
    permission_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteRolePermissionResponseCore {
    data: (),
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteRolePermissionResponse {
    response_message: String,
    response: Option<DeleteRolePermissionResponseCore>,
    error: Option<String>,
}

pub async fn delete_role_permission_controller(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    method: Method,
    uri: Uri,
    Json(payload): Json<DeleteRolePermissionRequest>,
) -> impl IntoResponse {
    let role_id = match Uuid::parse_str(&payload.role_id) {
        Ok(role_id) => role_id,
        Err(e) => {
            error!("INVALID ROLE ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(DeleteRolePermissionResponse {
                    response_message: "Failed to delete role permission".to_string(),
                    response: None,
                    error: Some("Invalid role id".to_string()),
                }),
            );
        }
    };

    let permission_id = match Uuid::parse_str(&payload.permission_id) {
        Ok(permission_id) => permission_id,
        Err(e) => {
            error!("INVALID PERMISSION ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(DeleteRolePermissionResponse {
                    response_message: "Failed to delete role permission".to_string(),
                    response: None,
                    error: Some("Invalid permission id".to_string()),
                }),
            );
        }
    };

    match delete_role_permission(&state.db, role_id, permission_id).await {
        Ok(_) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "delete_role_permission".to_string(),
                    activity_description: Some(
                        "Delete role permission endpoint accessed".to_string(),
                    ),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE DELETE ROLE PERMISSION SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DeleteRolePermissionResponse {
                        response_message: "Failed to delete role permission".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(DeleteRolePermissionResponse {
                    response_message: "Role permission deleted successfully".to_string(),
                    response: Some(DeleteRolePermissionResponseCore {
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
            error!("FAILED TO DELETE ROLE PERMISSION: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteRolePermissionResponse {
                    response_message: "Failed to delete role permission".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
