use crate::common::{authenticated_request, register_authenticated_user, setup_test_server_and_db};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_create_role_success() {
    let (server, db) = setup_test_server_and_db().await;
    let auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let role_name = format!("role_{}", unique_id);

    let response = authenticated_request(server.post("/api/v1/auth/roles"), &auth)
        .json(&serde_json::json!({
            "name": role_name,
            "description": "Test role"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);
    let body = response.json::<Value>();
    let role_id = body["response"]["data"]["id"].as_str().unwrap();
    assert_eq!(
        body["response"]["data"]["name"].as_str(),
        Some(role_name.as_str())
    );

    let role_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE id = $1::uuid")
        .bind(role_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(role_count, 1);
}
