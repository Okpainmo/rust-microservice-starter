use crate::common::{
    LoginRequest, RegisterRequest, TestLoginResponse, setup_test_server, setup_test_server_and_db,
};
use uuid::Uuid;

#[tokio::test]
async fn test_login_user_success() {
    let (server, db) = setup_test_server_and_db().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("login_{}@example.com", unique_id);
    let password = "secure_password123";

    server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Login".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: password.to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: email.clone(),
            password: password.to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<TestLoginResponse>();
    assert_eq!(body.response_message, "Login successful");
    let res = body.response.unwrap();
    assert!(!res.session_id.is_empty());
    assert!(res.access_token.is_some());
    assert!(res.refresh_token.is_some());

    let session_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id = $1::uuid")
            .bind(&res.session_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(session_count, 1);

    let sub_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'login'",
    )
    .bind(&res.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(sub_session_count, 1);

    let request_path: String = sqlx::query_scalar(
        "SELECT request_path FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'login'",
    )
    .bind(&res.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_path, "/login");

    let request_method: String = sqlx::query_scalar(
        "SELECT request_method FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'login'",
    )
    .bind(&res.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_method, "POST");

    let _ = response.cookie("auth_cookie");
}

#[tokio::test]
async fn test_login_user_invalid_password() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("wrong_pass_{}@example.com", unique_id);
    let password = "correct_password";

    server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Login".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: password.to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: email.clone(),
            password: "wrong_password".to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    let body = response.json::<TestLoginResponse>();
    assert_eq!(body.error.unwrap(), "Invalid email or password");
}

#[tokio::test]
async fn test_login_non_existent_user() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: "non_existent@example.com".to_string(),
            password: "any_password".to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}
