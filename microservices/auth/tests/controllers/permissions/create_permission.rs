use crate::common::{authenticated_request, register_authenticated_user, setup_test_server_and_db};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_create_permission_success() {
    let (server, db) = setup_test_server_and_db().await;
    let auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let permission_name = format!("permission_{}", unique_id);

    let response = authenticated_request(server.post("/api/v1/auth/permissions"), &auth)
        .json(&serde_json::json!({
            "name": permission_name,
            "description": "Test permission"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);
    let body = response.json::<Value>();
    let permission_id = body["response"]["data"]["id"].as_str().unwrap();
    assert_eq!(
        body["response"]["data"]["name"].as_str(),
        Some(permission_name.as_str())
    );

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permissions WHERE id = $1::uuid")
        .bind(permission_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
