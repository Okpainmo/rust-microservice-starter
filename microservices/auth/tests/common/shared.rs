use auth::db::connect_postgres::connect_pg;
use auth::middlewares::rate_limit_middleware::new_rate_limit_store;
use auth::utils::load_config::load_config;
use auth::{AppState, create_app};
use axum_test::{TestRequest, TestResponse, TestServer};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

const TEST_DB_MAX_CONNECTIONS: u32 = 1;

pub async fn setup_test_server_and_db() -> (TestServer, PgPool) {
    dotenvy::from_filename(".env.development").ok();

    let app_config = load_config().expect("Failed to load config");

    let db_config = app_config
        .database
        .as_ref()
        .expect("SERVER START-UP ERROR: DATABASE CONFIGURATION IS MISSING!");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config
            .user
            .as_deref()
            .expect("SERVER START-UP ERROR: DATABASE USER IS MISSING!"),
        db_config
            .password
            .as_deref()
            .expect("SERVER START-UP ERROR: DATABASE PASSWORD IS MISSING!"),
        db_config.host,
        db_config.port,
        db_config.name
    );

    let db_pool = connect_pg(
        database_url,
        TEST_DB_MAX_CONNECTIONS,
        db_config.connect_timeout_secs,
    )
    .await;

    let state = AppState {
        config: Arc::new(app_config),
        db: db_pool.clone(),
        rate_limit_store: new_rate_limit_store(),
    };

    let app = create_app(state);
    (
        TestServer::new(app).expect("Failed to create test server"),
        db_pool,
    )
}

pub async fn setup_test_server() -> TestServer {
    setup_test_server_and_db().await.0
}

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize)]
pub struct RegisterRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestRegisterResponse {
    pub response_message: String,
    pub response: Option<TestResponseCore>,
    pub error: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestLoginResponse {
    pub response_message: String,
    pub response: Option<TestResponseCore>,
    pub error: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestResponseCore {
    pub user_profile: Option<TestUserProfile>,
    pub session_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestUserProfile {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub user_type: String,
    pub country_code: Option<String>,
}

#[allow(dead_code)]
pub struct TestAuth {
    pub user_id: i64,
    pub email: String,
    pub session_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub auth_cookie: String,
}

pub async fn register_authenticated_user(server: &TestServer) -> TestAuth {
    let unique_id = Uuid::new_v4().to_string();
    let email = format!("auth_{}@example.com", unique_id);
    let password = "password123";

    let response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Auth".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: password.to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);
    let auth_cookie = response.cookie("auth_cookie").value().to_string();
    let body = response.json::<TestRegisterResponse>();
    let core = body.response.unwrap();
    let user_profile = core.user_profile.unwrap();

    TestAuth {
        user_id: user_profile.id,
        email,
        session_id: core.session_id,
        access_token: core.access_token.unwrap(),
        refresh_token: core.refresh_token.unwrap(),
        auth_cookie,
    }
}

pub async fn login_authenticated_user(
    server: &TestServer,
    email: &str,
    password: &str,
) -> TestAuth {
    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let auth_cookie = response.cookie("auth_cookie").value().to_string();
    let body = response.json::<TestLoginResponse>();
    let core = body.response.unwrap();
    let user_profile = core.user_profile.unwrap();

    TestAuth {
        user_id: user_profile.id,
        email: email.to_string(),
        session_id: core.session_id,
        access_token: core.access_token.unwrap(),
        refresh_token: core.refresh_token.unwrap(),
        auth_cookie,
    }
}

pub fn authenticated_request(request: TestRequest, auth: &TestAuth) -> TestRequest {
    request
        .add_header("user_id", auth.user_id.to_string())
        .authorization_bearer(&auth.access_token)
        .add_header("session_token", auth.refresh_token.clone())
        .add_header("session_id", auth.session_id.clone())
        .add_header("cookie", format!("auth_cookie={}", auth.auth_cookie))
}

pub fn refresh_auth_from_response(auth: &mut TestAuth, response: &TestResponse) {
    let body = response.json::<serde_json::Value>();
    let response = &body["response"];

    auth.session_id = response["session_id"]
        .as_str()
        .expect("response session_id should be present")
        .to_string();
    auth.access_token = response["access_token"]
        .as_str()
        .expect("response access_token should be present")
        .to_string();
    auth.refresh_token = response["refresh_token"]
        .as_str()
        .expect("response refresh_token should be present")
        .to_string();
}
