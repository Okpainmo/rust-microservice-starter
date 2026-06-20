use crate::common::{authenticated_request, register_authenticated_user, setup_test_server};
use serde_json::Value;

#[tokio::test]
async fn test_list_sessions_success() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let response = authenticated_request(server.get("/api/v1/auth/sessions"), &auth).await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    let sessions = body["response"]["data"].as_array().unwrap();
    assert!(
        sessions
            .iter()
            .any(|session| session["session"]["id"].as_str() == Some(auth.session_id.as_str()))
    );
}
