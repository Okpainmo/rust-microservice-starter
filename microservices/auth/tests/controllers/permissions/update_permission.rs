use crate::common::{
    authenticated_request, refresh_auth_from_response, register_authenticated_user,
    setup_test_server,
};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_update_permission_success() {
    let server = setup_test_server().await;
    let mut auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let permission_name = format!("permission_{}", unique_id);

    let permission_response = authenticated_request(server.post("/api/v1/auth/permissions"), &auth)
        .json(&serde_json::json!({
            "name": permission_name,
            "description": "Test permission"
        }))
        .await;
    permission_response.assert_status(axum::http::StatusCode::CREATED);
    refresh_auth_from_response(&mut auth, &permission_response);
    let permission_body = permission_response.json::<Value>();
    let permission_id = permission_body["response"]["data"]["id"].as_str().unwrap();

    let updated_permission_name = format!("updated_permission_{}", unique_id);
    let response = authenticated_request(
        server.patch(&format!("/api/v1/auth/permissions/{}", permission_id)),
        &auth,
    )
    .json(&serde_json::json!({
        "name": updated_permission_name,
        "description": "Updated test permission"
    }))
    .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert_eq!(
        body["response"]["data"]["name"].as_str().unwrap(),
        updated_permission_name
    );
}
