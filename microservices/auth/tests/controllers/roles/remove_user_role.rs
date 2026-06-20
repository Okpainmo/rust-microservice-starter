use crate::common::{
    authenticated_request, refresh_auth_from_response, register_authenticated_user,
    setup_test_server_and_db,
};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_remove_user_role_success() {
    let (server, db) = setup_test_server_and_db().await;
    let mut auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let role_name = format!("removable_role_{}", unique_id);

    let role_response = authenticated_request(server.post("/api/v1/auth/roles"), &auth)
        .json(&serde_json::json!({
            "name": role_name,
            "description": "Removable role"
        }))
        .await;
    role_response.assert_status(axum::http::StatusCode::CREATED);
    refresh_auth_from_response(&mut auth, &role_response);
    let role_body = role_response.json::<Value>();
    let role_id = role_body["response"]["data"]["id"].as_str().unwrap();

    let assign_response =
        authenticated_request(server.post("/api/v1/auth/roles/user/assign"), &auth)
            .json(&serde_json::json!({
                "user_id": auth.user_id,
                "role_id": role_id
            }))
            .await;
    assign_response.assert_status(axum::http::StatusCode::OK);
    refresh_auth_from_response(&mut auth, &assign_response);

    authenticated_request(server.post("/api/v1/auth/roles/user/remove"), &auth)
        .json(&serde_json::json!({
            "user_id": auth.user_id,
            "role_id": role_id
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2::uuid",
    )
    .bind(auth.user_id)
    .bind(role_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(count, 0);
}
