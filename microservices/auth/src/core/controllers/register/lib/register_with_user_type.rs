use crate::AppState;
use crate::core::services::role::user_role::assign_user_role_by_name;
use crate::core::services::session::create_session::{CreateSession, create_session};
use crate::core::services::sub_session::create_sub_session::{
    CreateSubSession, create_sub_session,
};
use crate::core::services::user::create_user::{CreateUser, create_user};
use crate::core::services::user::find_user::{find_user_by_email, find_user_by_phone_number};
use crate::core::services::user::update_user::{UpdateUser, update_user_by_id};
use crate::core::structs::user::RegisteredUserProfile;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::User;
use crate::utils::generate_tokens::generate_tokens;
use crate::utils::hashing_handler::hashing_handler;
use axum::Json;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
    country: Option<String>,
    country_code: Option<String>,
    phone_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResponseCore {
    user_profile: RegisteredUserProfile,
    session_id: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

pub struct RegisterActivity {
    pub activity_type: &'static str,
    pub entity_label: &'static str,
    pub request_method: String,
    pub request_path: String,
}

pub async fn register_with_user_type(
    cookies: Cookies,
    state: AppState,
    payload: RegisterRequest,
    user_type: &'static str,
    activity: RegisterActivity,
) -> (StatusCode, Json<RegisterResponse>) {
    let hashed_password = match hashing_handler(payload.password.as_str()).await {
        Ok(hash) => hash,
        Err(e) => {
            error!("PASSWORD HASHING ERROR!");

            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterResponse {
                    response_message: "Failed to hash password".to_string(),
                    response: None,
                    error: Some(format!("Password hashing error: {}", e)),
                }),
            );
        }
    };

    match find_user_by_email(&state.db, &payload.email).await {
        Ok(Some(_existing_user)) => {
            error!("REGISTRATION FAILED: EMAIL ALREADY EXISTS");

            return (
                StatusCode::FORBIDDEN,
                Json(RegisterResponse {
                    response_message: "Registration failed".to_string(),
                    response: None,
                    error: Some("Email already exists".to_string()),
                }),
            );
        }
        Ok(None) => {}
        Err(e) => {
            error!("ERROR WHILE CHECKING USER EMAIL UNIQUENESS: {}", e);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    response_message: "Registration failed".to_string(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    }

    if let Some(phone_number) = payload.phone_number.as_deref() {
        match find_user_by_phone_number(&state.db, phone_number).await {
            Ok(Some(_existing_user)) => {
                error!("REGISTRATION FAILED: PHONE NUMBER ALREADY EXISTS");

                return (
                    StatusCode::FORBIDDEN,
                    Json(RegisterResponse {
                        response_message: "Registration failed".to_string(),
                        response: None,
                        error: Some("Phone number already exists".to_string()),
                    }),
                );
            }
            Ok(None) => {}
            Err(e) => {
                error!("ERROR WHILE CHECKING USER PHONE NUMBER UNIQUENESS: {}", e);

                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterResponse {
                        response_message: "Registration failed".to_string(),
                        response: None,
                        error: Some(format!("Server error: {}", e)),
                    }),
                );
            }
        }
    }

    let full_name = format!("{} {}", payload.first_name, payload.last_name);

    let result = create_user(
        &state.db,
        CreateUser {
            email: payload.email.clone(),
            password: hashed_password,
            full_name,
            profile_image: "".to_string(),
            country: payload.country,
            country_code: payload.country_code,
            phone_number: payload.phone_number,
            user_type: user_type.to_string(),
        },
    )
    .await;

    match result {
        Ok(new_user) => {
            if let Err(e) = assign_user_role_by_name(&state.db, new_user.id, user_type).await {
                error!("FAILED TO ASSIGN USER ROLE: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterResponse {
                        response_message: "Registration failed".to_string(),
                        response: None,
                        error: Some(format!("Role assignment error: {}", e)),
                    }),
                );
            }

            let tokens = match generate_tokens(
                "auth",
                User {
                    id: new_user.id,
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
                        Json(RegisterResponse {
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
                            Json(RegisterResponse {
                                response_message: "Failed to hash access token".to_string(),
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
                            Json(RegisterResponse {
                                response_message: "Failed to hash refresh token".to_string(),
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
                        Json(RegisterResponse {
                            response_message: "Registration failed".to_string(),
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
                        Json(RegisterResponse {
                            response_message: "Registration failed".to_string(),
                            response: None,
                            error: Some("Auth configuration is missing".to_string()),
                        }),
                    );
                }
            };

            let session = match create_session(
                &state.db,
                CreateSession {
                    user_id: new_user.id,
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
                        Json(RegisterResponse {
                            response_message: "Registration failed".to_string(),
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
                    user_id: new_user.id,
                    activity_type: activity.activity_type.to_string(),
                    activity_description: Some(format!("{} registration", activity.entity_label)),
                    ip_address: None,
                    user_agent: None,
                    request_method: activity.request_method,
                    request_path: activity.request_path,
                },
            )
            .await
            {
                error!("FAILED TO CREATE REGISTER SUB-SESSION: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterResponse {
                        response_message: "Registration failed".to_string(),
                        response: None,
                        error: Some(format!("Sub-session creation error: {}", e)),
                    }),
                );
            }

            let update_result = update_user_by_id(
                &state.db,
                new_user.id,
                UpdateUser {
                    access_token: hashed_access_token,
                    refresh_token: hashed_refresh_token,
                    is_logged_out: None,
                },
            )
            .await;

            if let Err(e) = update_result {
                error!("FAILED TO UPDATE TOKENS: {}", e);
            }

            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap(), &state.config).await;

            (
                StatusCode::CREATED,
                Json(RegisterResponse {
                    response_message: format!(
                        "{} with email '{}' registered successfully!",
                        activity.entity_label, &payload.email
                    ),
                    response: Some(ResponseCore {
                        user_profile: new_user,
                        session_id: session.id.to_string(),
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            let error_msg =
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    error!("REGISTRATION FAILED: USER WITH EMAIL ALREADY EXIST!");
                    "Email already exists".to_string()
                } else {
                    error!("REGISTRATION FAILED: AN ERROR OCCURRED WHILE REGISTERING NEW USER!");
                    format!("Database error: {}", e)
                };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    response_message: "Failed to register user".to_string(),
                    response: None,
                    error: Some(error_msg),
                }),
            )
        }
    }
}
