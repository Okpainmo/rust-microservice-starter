use crate::AppState;
use crate::core::services::role::role_permission::assign_role_permission;
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
pub struct AssignRolePermissionRequest {
    role_id: String,
    permission_id: String,
}

#[derive(Debug, Serialize)]
pub struct AssignRolePermissionResponseCore {
    data: (),
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AssignRolePermissionResponse {
    response_message: String,
    response: Option<AssignRolePermissionResponseCore>,
    error: Option<String>,
}

pub async fn assign_role_permission_controller(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    method: Method,
    uri: Uri,
    Json(payload): Json<AssignRolePermissionRequest>,
) -> impl IntoResponse {
    let role_id = match Uuid::parse_str(&payload.role_id) {
        Ok(role_id) => role_id,
        Err(e) => {
            error!("INVALID ROLE ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(AssignRolePermissionResponse {
                    response_message: "Failed to assign permission".to_string(),
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
                Json(AssignRolePermissionResponse {
                    response_message: "Failed to assign permission".to_string(),
                    response: None,
                    error: Some("Invalid permission id".to_string()),
                }),
            );
        }
    };

    match assign_role_permission(&state.db, role_id, permission_id).await {
        Ok(_) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "assign_role_permission".to_string(),
                    activity_description: Some(
                        "Assign role permission endpoint accessed".to_string(),
                    ),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE ASSIGN ROLE PERMISSION SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AssignRolePermissionResponse {
                        response_message: "Failed to assign permission".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(AssignRolePermissionResponse {
                    response_message: "Permission assigned successfully".to_string(),
                    response: Some(AssignRolePermissionResponseCore {
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
            error!("FAILED TO ASSIGN ROLE PERMISSION: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AssignRolePermissionResponse {
                    response_message: "Failed to assign permission".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
