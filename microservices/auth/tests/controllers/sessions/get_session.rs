use crate::common::{
    authenticated_request, login_authenticated_user, register_authenticated_user, setup_test_server,
};
use serde_json::Value;

#[tokio::test]
async fn test_get_session_success() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let response = authenticated_request(
        server.get(&format!("/api/v1/auth/sessions/{}", auth.session_id)),
        &auth,
    )
    .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert_eq!(
        body["response"]["data"]["session"]["id"].as_str().unwrap(),
        auth.session_id
    );
    assert_eq!(
        body["response"]["data"]["session"]["sub_sessions"][0]["activity_type"]
            .as_str()
            .unwrap(),
        "register"
    );
    assert!(
        body["response"]["data"]["session"]["refresh_token_hash"].is_null(),
        "session responses must not expose refresh token hashes"
    );
}

#[tokio::test]
async fn test_get_session_rejects_mismatched_session_token() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;
    let second_auth = login_authenticated_user(&server, &auth.email, "password123").await;

    let response = server
        .get(&format!("/api/v1/auth/sessions/{}", auth.session_id))
        .add_header("user_id", auth.user_id.to_string())
        .authorization_bearer(&auth.access_token)
        .add_header("session_token", second_auth.refresh_token)
        .add_header("session_id", auth.session_id.clone())
        .add_header("cookie", format!("auth_cookie={}", auth.auth_cookie))
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    let body = response.json::<Value>();
    assert_eq!(
        body["response_message"].as_str().unwrap(),
        "Session token does not match active session"
    );
}

#[tokio::test]
async fn test_get_session_rejects_invalid_bearer_token() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let response = server
        .get(&format!("/api/v1/auth/sessions/{}", auth.session_id))
        .add_header("user_id", auth.user_id.to_string())
        .authorization_bearer("not-a-valid-access-token")
        .add_header("session_token", auth.refresh_token.clone())
        .add_header("session_id", auth.session_id.clone())
        .add_header("cookie", format!("auth_cookie={}", auth.auth_cookie))
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}
