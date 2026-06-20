use crate::AppState;
use crate::core::services::permission::find_permission::find_permissions_by_user_id;
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
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ListUserPermissionsResponseCore {
    data: Vec<Permission>,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct ListUserPermissionsResponse {
    response_message: String,
    response: Option<ListUserPermissionsResponseCore>,
    error: Option<String>,
}

pub async fn list_user_permissions(
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
                Json(ListUserPermissionsResponse {
                    response_message: "Failed to list user permissions".to_string(),
                    response: None,
                    error: Some("Invalid user id".to_string()),
                }),
            );
        }
    };

    match find_permissions_by_user_id(&state.db, user_id).await {
        Ok(permissions) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "list_user_permissions".to_string(),
                    activity_description: Some(
                        "List user permissions endpoint accessed".to_string(),
                    ),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE LIST USER PERMISSIONS SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ListUserPermissionsResponse {
                        response_message: "Failed to list user permissions".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(ListUserPermissionsResponse {
                    response_message: "User permissions fetched successfully".to_string(),
                    response: Some(ListUserPermissionsResponseCore {
                        data: permissions,
                        session_id: session_output.session.id.to_string(),
                        access_token: session_output.access_token,
                        refresh_token: session_output.session_token,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("FAILED TO LIST USER PERMISSIONS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ListUserPermissionsResponse {
                    response_message: "Failed to list user permissions".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
