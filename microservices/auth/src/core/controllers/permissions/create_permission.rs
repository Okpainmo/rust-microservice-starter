use crate::AppState;
use crate::core::services::permission::create_permission::{CreatePermission, create_permission};
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::structs::permission::Permission;
use crate::middlewares::sessions_middleware::SessionsMiddlewareOutput;
use axum::extract::{Extension, State};
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePermissionResponseCore {
    data: Permission,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePermissionResponse {
    response_message: String,
    response: Option<CreatePermissionResponseCore>,
    error: Option<String>,
}

pub async fn create_permission_controller(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    method: Method,
    uri: Uri,
    Json(payload): Json<CreatePermissionRequest>,
) -> impl IntoResponse {
    match create_permission(
        &state.db,
        CreatePermission {
            name: payload.name,
            description: payload.description,
        },
    )
    .await
    {
        Ok(permission) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "create_permission".to_string(),
                    activity_description: Some("Create permission endpoint accessed".to_string()),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE PERMISSION SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CreatePermissionResponse {
                        response_message: "Failed to create permission".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::CREATED,
                Json(CreatePermissionResponse {
                    response_message: "Permission created successfully".to_string(),
                    response: Some(CreatePermissionResponseCore {
                        data: permission,
                        session_id: session_output.session.id.to_string(),
                        access_token: session_output.access_token,
                        refresh_token: session_output.session_token,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("FAILED TO CREATE PERMISSION: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CreatePermissionResponse {
                    response_message: "Failed to create permission".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
