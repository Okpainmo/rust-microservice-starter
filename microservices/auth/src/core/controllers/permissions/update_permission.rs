use crate::AppState;
use crate::core::services::permission::update_permission::{UpdatePermission, update_permission};
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::structs::permission::Permission;
use crate::middlewares::sessions_middleware::SessionsMiddlewareOutput;
use axum::extract::{Extension, Path, State};
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdatePermissionResponseCore {
    data: Permission,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct UpdatePermissionResponse {
    response_message: String,
    response: Option<UpdatePermissionResponseCore>,
    error: Option<String>,
}

pub async fn update_permission_controller(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    Path(permission_id): Path<String>,
    method: Method,
    uri: Uri,
    Json(payload): Json<UpdatePermissionRequest>,
) -> impl IntoResponse {
    let permission_id = match Uuid::parse_str(&permission_id) {
        Ok(permission_id) => permission_id,
        Err(e) => {
            error!("INVALID PERMISSION ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(UpdatePermissionResponse {
                    response_message: "Failed to update permission".to_string(),
                    response: None,
                    error: Some("Invalid permission id".to_string()),
                }),
            );
        }
    };

    match update_permission(
        &state.db,
        permission_id,
        UpdatePermission {
            name: payload.name,
            description: payload.description,
        },
    )
    .await
    {
        Ok(Some(permission)) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "update_permission".to_string(),
                    activity_description: Some("Update permission endpoint accessed".to_string()),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE UPDATE PERMISSION SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UpdatePermissionResponse {
                        response_message: "Failed to update permission".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(UpdatePermissionResponse {
                    response_message: "Permission updated successfully".to_string(),
                    response: Some(UpdatePermissionResponseCore {
                        data: permission,
                        session_id: session_output.session.id.to_string(),
                        access_token: session_output.access_token,
                        refresh_token: session_output.session_token,
                    }),
                    error: None,
                }),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(UpdatePermissionResponse {
                response_message: "Failed to update permission".to_string(),
                response: None,
                error: Some("Permission not found".to_string()),
            }),
        ),
        Err(e) => {
            error!("FAILED TO UPDATE PERMISSION: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdatePermissionResponse {
                    response_message: "Failed to update permission".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
