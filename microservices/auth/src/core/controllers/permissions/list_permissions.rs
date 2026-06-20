use crate::AppState;
use crate::core::services::permission::find_permission::find_all_permissions;
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
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ListPermissionsResponseCore {
    data: Vec<Permission>,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct ListPermissionsResponse {
    response_message: String,
    response: Option<ListPermissionsResponseCore>,
    error: Option<String>,
}

pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(session_output): Extension<SessionsMiddlewareOutput>,
    method: Method,
    uri: Uri,
) -> impl IntoResponse {
    match find_all_permissions(&state.db).await {
        Ok(permissions) => {
            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session_output.session.id,
                    user_id: session_output.user.id,
                    activity_type: "list_permissions".to_string(),
                    activity_description: Some("List permissions endpoint accessed".to_string()),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE LIST PERMISSIONS SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ListPermissionsResponse {
                        response_message: "Failed to list permissions".to_string(),
                        response: None,
                        error: Some(e.to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(ListPermissionsResponse {
                    response_message: "Permissions fetched successfully".to_string(),
                    response: Some(ListPermissionsResponseCore {
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
            error!("FAILED TO LIST PERMISSIONS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ListPermissionsResponse {
                    response_message: "Failed to list permissions".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}
