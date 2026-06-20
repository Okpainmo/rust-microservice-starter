use chat__auth_server::{create_app, AppState};
use chat__auth_server::utils::load_config::load_config;
use chat__auth_server::db::connect_postgres::connect_pg;
use axum_test::TestServer;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_register_user_success() {
    // Load environment variables for the test (make sure you have a .env.development or similar)
    dotenvy::from_filename(".env.development").ok();

    let app_config = load_config().expect("Failed to load config");

    let db_config = app_config.database.as_ref().expect("SERVER START-UP ERROR: DATABASE CONFIGURATION IS MISSING!");
    
     let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config.user.as_deref().expect("SERVER START-UP ERROR: DATABASE USER IS MISSING!"),
        db_config.password.as_deref().expect("SERVER START-UP ERROR: DATABASE PASSWORD IS MISSING!"),
        db_config.host,
        db_config.port,
        db_config.name
    );

    let db_pool = connect_pg(database_url).await;

    let state = AppState {
        config: Arc::new(app_config),
        db: db_pool,
    };

    let app = create_app(state);
    let server = TestServer::new(app).expect("Failed to create test server");

    // Generate a unique email for each test run to avoid "Email already exists" errors
    let unique_id = uuid::Uuid::new_v4().to_string();
    let email = format!("test_{}@example.com", unique_id);

    let response = server
        .post("/api/v1/auth/register")
        .json(&json!({
            "first_name": "Test",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "country": "TestCountry",
            "phone_number": unique_id[0..8].to_string(),
        }))
        .await;

    // response.assert_status_success();
    response.assert_status(axum::http::StatusCode::CREATED);
}
