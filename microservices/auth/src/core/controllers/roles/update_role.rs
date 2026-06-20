use crate::AppState;
use crate::core::services::role::update_role::{UpdateRole, update_role};
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::structs::role::Role;
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
pub struct UpdateRoleRequest {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateRoleResponseCore {
    data: Role,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateRoleResponse {
    response_message: String,
    response: Option<UpdateRoleResponseCore>,
    error: Option<String>,
}

pub async fn update_role_controller(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    Path(role_id): Path<String>,
    method: Method,
    uri: Uri,
    Json(payload): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    let role_id = match Uuid::parse_str(&role_id) {
        Ok(role_id) => role_id,
        Err(e) => {
            error!("INVALID ROLE ID: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(UpdateRoleResponse {
                    response_message: "Failed to update role".to_string(),
                    response: None,
                    error: Some("Invalid role id".to_string()),
                }),
            );
        }
    };

    match update_role(
        &state.db,
        role_id,
        UpdateRole {
            name: payload.name,
            description: payload.description,
        },
    )
    .await
    {
        Ok(Some(role)) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "update_role".to_string(),
                    activity_description: Some("Update role endpoint accessed".to_string()),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE UPDATE ROLE SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UpdateRoleResponse {
                        response_message: "Failed to update role".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(UpdateRoleResponse {
                    response_message: "Role updated successfully".to_string(),
                    response: Some(UpdateRoleResponseCore {
                        data: role,
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
            Json(UpdateRoleResponse {
                response_message: "Failed to update role".to_string(),
                response: None,
                error: Some("Role not found".to_string()),
            }),
        ),
        Err(e) => {
            error!("FAILED TO UPDATE ROLE: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateRoleResponse {
                    response_message: "Failed to update role".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
