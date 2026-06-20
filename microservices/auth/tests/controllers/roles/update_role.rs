use crate::common::{
    authenticated_request, refresh_auth_from_response, register_authenticated_user,
    setup_test_server,
};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_update_role_success() {
    let server = setup_test_server().await;
    let mut auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let role_name = format!("role_{}", unique_id);

    let role_response = authenticated_request(server.post("/api/v1/auth/roles"), &auth)
        .json(&serde_json::json!({
            "name": role_name,
            "description": "Test role"
        }))
        .await;

    role_response.assert_status(axum::http::StatusCode::CREATED);
    refresh_auth_from_response(&mut auth, &role_response);
    let role_body = role_response.json::<Value>();
    let role_id = role_body["response"]["data"]["id"].as_str().unwrap();
    let updated_role_name = format!("updated_role_{}", unique_id);

    let response = authenticated_request(
        server.patch(&format!("/api/v1/auth/roles/{}", role_id)),
        &auth,
    )
    .json(&serde_json::json!({
        "name": updated_role_name,
        "description": "Updated test role"
    }))
    .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert_eq!(
        body["response"]["data"]["name"].as_str().unwrap(),
        updated_role_name
    );
}
