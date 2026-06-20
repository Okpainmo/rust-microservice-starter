use crate::AppState;
use crate::core::services::session::create_session::{CreateSession, create_session};
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::services::user::find_user::find_user_profile_by_email;
use crate::core::services::user::update_user::{UpdateUser, update_user_by_email};
use crate::core::structs::user::UserProfile;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::{User, generate_tokens};
use crate::utils::hashing_handler::hashing_handler;
use crate::utils::verification_handler::verification_handler; // your existing password verification function
use axum::extract::State;
use axum::{
    Json,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ResponseCore {
    user_profile: UserProfile,
    session_id: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

// Reuse UserProfile and ResponseCore from register controller

pub async fn login_user(
    cookies: Cookies,
    // Extension(db_pool): Extension<PgPool>,
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Fetch user by email
    let user_result = find_user_profile_by_email(&state.db, &payload.email).await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("LOGIN FAILED: PROVIDE EMAIL AND PASSWORD!");

            return (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some("Invalid email or password".to_string()),
                }),
            );
        }
        Err(e) => {
            error!("USER LOGIN FAILED!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    };

    match verification_handler(&payload.password, &user.password).await {
        Ok(true) => {
            let tokens = match generate_tokens(
                "auth",
                User {
                    id: user.id,
                    email: payload.email.clone(),
                },
                &state.config,
            )
            .await
            {
                Ok(tokens) => tokens,
                Err(e) => {
                    error!("TOKEN GENERATION ERROR!");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse {
                            response_message: "Failed to generate tokens".to_string(),
                            response: None,
                            error: Some(format!("Token generation error: {}", e)),
                        }),
                    );
                }
            };

            let hashed_access_token = match tokens.access_token.as_deref() {
                Some(access_token) => match hashing_handler(access_token).await {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("ACCESS TOKEN HASHING ERROR!");
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(LoginResponse {
                                response_message: "Login failed".to_string(),
                                response: None,
                                error: Some(format!("Access token hashing error: {}", e)),
                            }),
                        );
                    }
                },
                None => None,
            };

            let hashed_refresh_token = match tokens.refresh_token.as_deref() {
                Some(refresh_token) => match hashing_handler(refresh_token).await {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("REFRESH TOKEN HASHING ERROR!");
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(LoginResponse {
                                response_message: "Login failed".to_string(),
                                response: None,
                                error: Some(format!("Refresh token hashing error: {}", e)),
                            }),
                        );
                    }
                },
                None => None,
            };

            let refresh_token_hash = match hashed_refresh_token.clone() {
                Some(refresh_token_hash) => refresh_token_hash,
                None => {
                    error!("REFRESH TOKEN WAS NOT GENERATED!");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse {
                            response_message: "Login failed".to_string(),
                            response: None,
                            error: Some("Refresh token was not generated".to_string()),
                        }),
                    );
                }
            };

            let auth = match state.config.auth.as_ref() {
                Some(auth) => auth,
                None => {
                    error!("AUTH CONFIGURATION IS MISSING!");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse {
                            response_message: "Login failed".to_string(),
                            response: None,
                            error: Some("Auth configuration is missing".to_string()),
                        }),
                    );
                }
            };

            let session = match create_session(
                &state.db,
                CreateSession {
                    user_id: user.id,
                    refresh_token_hash,
                    expires_at: (Utc::now()
                        + Duration::hours(auth.jwt_refresh_expiration_time_in_hours as i64))
                    .naive_utc(),
                },
            )
            .await
            {
                Ok(session) => session,
                Err(e) => {
                    error!("FAILED TO CREATE SESSION: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse {
                            response_message: "Login failed".to_string(),
                            response: None,
                            error: Some(format!("Session creation error: {}", e)),
                        }),
                    );
                }
            };

            if let Err(e) = create_sub_session(
                &state.db,
                CreateSubSession {
                    session_id: session.id,
                    user_id: user.id,
                    activity_type: "login".to_string(),
                    activity_description: Some("User log in".to_string()),
                    ip_address: None,
                    user_agent: None,
                    request_method: method.as_str().to_string(),
                    request_path: uri.path().to_string(),
                },
            )
            .await
            {
                error!("FAILED TO CREATE LOGIN SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoginResponse {
                        response_message: "Login failed".to_string(),
                        response: None,
                        error: Some(format!("Sub-session creation error: {}", e)),
                    }),
                );
            }

            let _ = update_user_by_email(
                &state.db,
                &payload.email,
                UpdateUser {
                    access_token: hashed_access_token,
                    refresh_token: hashed_refresh_token,
                    is_logged_out: Some(false),
                },
            )
            .await;

            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap(), &state.config).await;

            (
                StatusCode::OK,
                Json(LoginResponse {
                    response_message: "Login successful".to_string(),
                    response: Some(ResponseCore {
                        user_profile: UserProfile {
                            is_logged_out: false,
                            ..user
                        },
                        session_id: session.id.to_string(),
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                    }),
                    error: None,
                }),
            )
        }
        Ok(false) => {
            error!("USER LOGIN FAILED!");

            (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some("Invalid email or password".to_string()),
                }),
            )
        }
        Err(e) => {
            error!("USER LOGIN FAILED!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some(format!("Password verification error: {}", e)),
                }),
            )
        }
    }
}
