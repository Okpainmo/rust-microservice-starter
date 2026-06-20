use crate::AppState;
use crate::core::controllers::register::lib::register_with_user_type::{
    RegisterActivity, RegisterRequest, register_with_user_type,
};
use axum::extract::State;
use axum::{
    Json,
    http::{Method, Uri},
    response::IntoResponse,
};
use tower_cookies::Cookies;

pub async fn register_user(
    cookies: Cookies,
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    register_with_user_type(
        cookies,
        state,
        payload,
        "user",
        RegisterActivity {
            activity_type: "register",
            entity_label: "User",
            request_method: method.as_str().to_string(),
            request_path: uri.path().to_string(),
        },
    )
    .await
}
