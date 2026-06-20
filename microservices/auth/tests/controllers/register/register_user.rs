use crate::common::{
    RegisterRequest, TestRegisterResponse, setup_test_server, setup_test_server_and_db,
};
use uuid::Uuid;

#[tokio::test]
async fn test_register_user_success() {
    let (server, db) = setup_test_server_and_db().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("test_{}@example.com", unique_id);
    let phone = unique_id[0..10].to_string();

    let response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(phone),
        })
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);

    let body = response.json::<TestRegisterResponse>();
    assert_eq!(
        body.response_message,
        format!("User with email '{}' registered successfully!", email)
    );
    let response = body.response.unwrap();
    assert!(!response.session_id.is_empty());
    let user_profile = response.user_profile.unwrap();
    assert!(user_profile.id > 0);
    assert_eq!(user_profile.user_type, "user");
    assert_eq!(user_profile.country_code.unwrap(), "TC");

    let session_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id = $1::uuid")
            .bind(&response.session_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(session_count, 1);

    let sub_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'register'",
    )
    .bind(&response.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(sub_session_count, 1);

    let request_path: String = sqlx::query_scalar(
        "SELECT request_path FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'register'",
    )
    .bind(&response.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_path, "/register");

    let request_method: String = sqlx::query_scalar(
        "SELECT request_method FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'register'",
    )
    .bind(&response.session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_method, "POST");
}

#[tokio::test]
async fn test_register_user_with_required_fields_only() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("required_only_{}@example.com", unique_id);

    let response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Required".to_string(),
            last_name: "Only".to_string(),
            email: email.clone(),
            password: "password123".to_string(),
            country: None,
            country_code: None,
            phone_number: None,
        })
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);

    let body = response.json::<TestRegisterResponse>();
    assert_eq!(
        body.response_message,
        format!("User with email '{}' registered successfully!", email)
    );
    let user_profile = body.response.unwrap().user_profile.unwrap();
    assert_eq!(user_profile.user_type, "user");
    assert!(user_profile.country_code.is_none());
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("dup_{}@example.com", unique_id);
    let phone1 = unique_id[0..10].to_string();
    let phone2 = unique_id[11..21].to_string();

    server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(phone1),
        })
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Test2".to_string(),
            last_name: "User2".to_string(),
            email,
            password: "password456".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(phone2),
        })
        .await;

    response.assert_status(axum::http::StatusCode::FORBIDDEN);
    let body = response.json::<TestRegisterResponse>();
    assert_eq!(body.error.unwrap(), "Email already exists");
}

#[tokio::test]
async fn test_register_user_duplicate_phone_number() {
    let server = setup_test_server().await;

    let unique_id1 = Uuid::new_v4().to_string();
    let email1 = format!("phone_dup1_{}@example.com", unique_id1);
    let phone = unique_id1[0..10].to_string();

    let unique_id2 = Uuid::new_v4().to_string();
    let email2 = format!("phone_dup2_{}@example.com", unique_id2);

    server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: email1,
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(phone.clone()),
        })
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Test2".to_string(),
            last_name: "User2".to_string(),
            email: email2,
            password: "password456".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(phone),
        })
        .await;

    response.assert_status(axum::http::StatusCode::FORBIDDEN);
    let body = response.json::<TestRegisterResponse>();
    assert_eq!(body.error.unwrap(), "Phone number already exists");
}
