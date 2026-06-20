use crate::common::{RegisterRequest, TestRegisterResponse, setup_test_server_and_db};
use uuid::Uuid;

#[tokio::test]
async fn test_register_admin_success() {
    let (server, db) = setup_test_server_and_db().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("admin_{}@example.com", unique_id);
    let phone = unique_id[0..10].to_string();

    let response = server
        .post("/api/v1/auth/register/admin")
        .json(&RegisterRequest {
            first_name: "Admin".to_string(),
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
        format!("Admin with email '{}' registered successfully!", email)
    );
    let response = body.response.unwrap();
    let user_profile = response.user_profile.unwrap();
    assert_eq!(user_profile.user_type, "admin");

    let user_role_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = $1 AND r.name = 'admin'
        "#,
    )
    .bind(user_profile.id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(user_role_count, 1);
}
