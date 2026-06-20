use crate::common::{authenticated_request, register_authenticated_user, setup_test_server};
use chrono::{Duration, Utc};
use serde_json::Value;

#[tokio::test]
async fn test_update_session_success() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let expires_at = Utc::now() + Duration::hours(48);
    let response = authenticated_request(
        server.patch(&format!("/api/v1/auth/sessions/{}", auth.session_id)),
        &auth,
    )
    .json(&serde_json::json!({
        "expires_at_in_milliseconds": expires_at.timestamp_millis(),
    }))
    .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert_eq!(
        body["response"]["data"]["session"]["id"].as_str().unwrap(),
        auth.session_id
    );
}
