use crate::common::{
    TestLoginResponse, authenticated_request, register_authenticated_user, setup_test_server,
    setup_test_server_and_db,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct LogoutRequest {
    user_email: String,
    session_id: String,
}

#[tokio::test]
async fn test_logout_user_success() {
    let (server, db) = setup_test_server_and_db().await;
    let auth = register_authenticated_user(&server).await;

    let response = authenticated_request(server.post("/api/v1/auth/logout"), &auth)
        .json(&LogoutRequest {
            user_email: auth.email.clone(),
            session_id: auth.session_id.clone(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<TestLoginResponse>();
    assert_eq!(body.response_message, "Logout successful");

    let revoked_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE id = $1::uuid AND status = 'revoked' AND revoked_at IS NOT NULL",
    )
    .bind(&auth.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(revoked_count, 1);

    let sub_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'logout'",
    )
    .bind(&auth.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(sub_session_count, 1);

    let request_path: String = sqlx::query_scalar(
        "SELECT request_path FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'logout'",
    )
    .bind(&auth.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_path, "/logout");

    let request_method: String = sqlx::query_scalar(
        "SELECT request_method FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'logout'",
    )
    .bind(&auth.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_method, "POST");
}

#[tokio::test]
async fn test_logout_non_existent_user() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let response = authenticated_request(server.post("/api/v1/auth/logout"), &auth)
        .json(&LogoutRequest {
            user_email: "ghost@example.com".to_string(),
            session_id: Uuid::new_v4().to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}
