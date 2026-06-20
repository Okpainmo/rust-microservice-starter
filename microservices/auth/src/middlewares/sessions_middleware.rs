use crate::core::services::session::find_sessions::find_session_by_id;
use crate::core::services::session::update_session::renew_session;
use crate::core::services::user::find_user::find_user_profile_by_id;
use crate::core::structs::session::Session;
use crate::core::structs::user::UserProfile;
use crate::utils::generate_tokens::{User, generate_tokens};
use crate::utils::hashing_handler::hashing_handler;
use crate::utils::verification_handler::verification_handler;
use axum::extract::State;
use axum::{
    Json,
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, Validation, decode, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub response_message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
    OneTimePassword,
}

#[derive(Debug, Deserialize)]
pub struct JwtClaims {
    pub id: i64,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub token_kind: Option<TokenType>,
}

#[derive(Clone)]
pub struct MiddlewareState {
    pub jwt_secret: String,
    pub cookie_name: String,
}

#[derive(Clone, Debug)]
pub struct SessionsMiddlewareOutput {
    pub user: UserProfile,
    pub session_status: String,
    pub session: Session,
    pub access_token: String,
    pub session_token: String,
}

// ============================================================================
// Sessions Middleware
// ============================================================================

pub async fn sessions_middleware(
    // Extension(db_pool): Extension<PgPool>,
    State(state): State<crate::AppState>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let auth_config = match state.config.auth.as_ref() {
        Some(config) => config,
        None => {
            error!("AUTH CONFIGURATION MISSING!");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Server Error".to_string(),
                    response_message: "Auth configuration is missing".to_string(),
                }),
            ));
        }
    };

    let session_state = MiddlewareState {
        jwt_secret: auth_config.jwt_secret.clone(),
        cookie_name: "auth_cookie".to_string(),
    };

    // ------------------------------------------------------------------------
    // Extract required headers
    // ------------------------------------------------------------------------
    let user_id = req
        .headers()
        .get("user_id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("USER ID HEADER MISSING!");

            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "User id header missing".to_string(),
                }),
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            error!("INVALID USER ID HEADER!");

            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "User id header is invalid".to_string(),
                }),
            )
        })?;

    let _authorization = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("AUTHORIZATION HEADER MISSING!");

            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Authorization header missing".to_string(),
                }),
            )
        })?;

    let session_token = req
        .headers()
        .get("session_token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("SESSION TOKEN HEADER MISSING!");

            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Session token header missing".to_string(),
                }),
            )
        })?;

    let session_token = session_token.trim();

    let session_id = req
        .headers()
        .get("session_id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("SESSION ID HEADER MISSING!");

            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Session id header missing".to_string(),
                }),
            )
        })?;

    let session_id = Uuid::parse_str(session_id).map_err(|_| {
        error!("INVALID SESSION ID HEADER!");

        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                response_message: "Session id header is invalid".to_string(),
            }),
        )
    })?;

    // ------------------------------------------------------------------------
    // Validate cookie presence
    // ------------------------------------------------------------------------
    let auth_cookie = match cookies.get(&session_state.cookie_name) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            error!("AUTH COOKIE NOT FOUND!");

            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Request rejected, please re-authenticate".to_string(),
                }),
            ));
        }
    };

    // ------------------------------------------------------------------------
    // Fetch user from database
    // ------------------------------------------------------------------------
    let user = match find_user_profile_by_id(&state.db, user_id).await {
        Ok(Some(u)) => u,

        Ok(None) => {
            error!("USER NOT FOUND!");

            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Not Found".to_string(),
                    response_message: format!("User with id '{}' not found", user_id),
                }),
            ));
        }

        Err(e) => {
            error!("USER FETCH FAILED!");

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DB Error".to_string(),
                    response_message: e.to_string(),
                }),
            ));
        }
    };

    // ------------------------------------------------------------------------
    // Check active status
    // ------------------------------------------------------------------------
    if !user.is_active {
        error!("INACTIVE USER ACCESS BLOCKED!");

        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Forbidden".to_string(),
                response_message: "Your account is deactivated".to_string(),
            }),
        ));
    }

    // ------------------------------------------------------------------------
    // Verify auth cookie belongs to the resolved user
    // ------------------------------------------------------------------------
    let cookie_email_hash = match auth_cookie.split_once("____") {
        Some(("auth_cookie", hashed_email)) if !hashed_email.trim().is_empty() => hashed_email,

        _ => {
            error!("INVALID AUTH COOKIE FORMAT!");

            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Invalid auth cookie".to_string(),
                }),
            ));
        }
    };

    match verification_handler(&user.email, cookie_email_hash).await {
        Ok(true) => {}

        Ok(false) => {
            error!("AUTH COOKIE USER MISMATCH!");

            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Auth cookie does not match user".to_string(),
                }),
            ));
        }

        Err(e) => {
            error!("AUTH COOKIE VERIFICATION FAILED!");

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Auth Cookie Verification Failed".to_string(),
                    response_message: e.to_string(),
                }),
            ));
        }
    }

    // ------------------------------------------------------------------------
    // Validate refresh/session JWT
    // ------------------------------------------------------------------------
    let decoding_key = DecodingKey::from_secret(session_state.jwt_secret.as_bytes());

    match decode::<JwtClaims>(session_token, &decoding_key, &Validation::default()) {
        Ok(token_data) => {
            if token_data.claims.id != user.id {
                error!("USER ID CLAIM MISMATCH!");

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "User credentials do not match".to_string(),
                    }),
                ));
            }

            if token_data.claims.email != user.email {
                error!("USER EMAIL CLAIM MISMATCH!");

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "User credentials do not match".to_string(),
                    }),
                ));
            }

            if token_data
                .claims
                .token_kind
                .is_some_and(|token_kind| token_kind != TokenType::Refresh)
            {
                error!("INVALID TOKEN TYPE CLAIM!");

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "Invalid token type".to_string(),
                    }),
                ));
            }

            // ------------------------------------------------------------------------
            // Find and renew session since the session token is yet to expire
            // ------------------------------------------------------------------------
            let session = match find_session_by_id(&state.db, session_id).await {
                Ok(Some(session)) => session,

                Err(e) => {
                    error!("SESSION FETCH FAILED!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "DB Error".to_string(),
                            response_message: e.to_string(),
                        }),
                    ));
                }

                Ok(None) => {
                    error!("SESSION NOT FOUND!");

                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse {
                            error: "Not Found".to_string(),
                            response_message: "Session not found".to_string(),
                        }),
                    ));
                }
            };

            if session.user_id != user.id {
                error!("SESSION USER MISMATCH!");

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "Session does not belong to authenticated user"
                            .to_string(),
                    }),
                ));
            }

            if session.status == "revoked" {
                error!("REVOKED SESSION ACCESS BLOCKED!");

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "Session has been revoked, please re-authenticate"
                            .to_string(),
                    }),
                ));
            }

            match verification_handler(session_token, &session.refresh_token_hash).await {
                Ok(true) => {}

                Ok(false) => {
                    error!("SESSION TOKEN HASH MISMATCH!");

                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorResponse {
                            error: "Unauthorized".to_string(),
                            response_message: "Session token does not match active session"
                                .to_string(),
                        }),
                    ));
                }

                Err(e) => {
                    error!("SESSION TOKEN VERIFICATION FAILED!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Session Token Verification Failed".to_string(),
                            response_message: e.to_string(),
                        }),
                    ));
                }
            }

            let session_expiry =
                match i64::try_from(auth_config.jwt_refresh_expiration_time_in_hours) {
                    Ok(session_expiry) => session_expiry,
                    Err(_) => {
                        error!("INVALID SESSION EXPIRATION CONFIGURATION!");

                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Server Error".to_string(),
                                response_message: "Invalid session expiration configuration"
                                    .to_string(),
                            }),
                        ));
                    }
                };

            let renewed_expires_at =
                match Utc::now().checked_add_signed(Duration::hours(session_expiry)) {
                    Some(expires_at) => expires_at.naive_utc(),
                    None => {
                        error!("SESSION EXPIRATION RENEWAL FAILED!");

                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Server Error".to_string(),
                                response_message: "Failed to renew user session".to_string(),
                            }),
                        ));
                    }
                };

            let tokens = match generate_tokens(
                "auth",
                User {
                    id: user.id,
                    email: user.email.clone(),
                },
                &state.config,
            )
            .await
            {
                Ok(tokens) => tokens,
                Err(e) => {
                    error!("TOKEN GENERATION ERROR!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Token Generation Error".to_string(),
                            response_message: e.to_string(),
                        }),
                    ));
                }
            };

            let new_access_token = match tokens.access_token {
                Some(access_token) => access_token,
                None => {
                    error!("ACCESS TOKEN WAS NOT GENERATED!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Token Generation Error".to_string(),
                            response_message: "Access token was not generated".to_string(),
                        }),
                    ));
                }
            };

            let new_session_token = match tokens.refresh_token {
                Some(refresh_token) => refresh_token,
                None => {
                    error!("SESSION TOKEN WAS NOT GENERATED!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Token Generation Error".to_string(),
                            response_message: "Session token was not generated".to_string(),
                        }),
                    ));
                }
            };

            let new_session_token_hash = match hashing_handler(&new_session_token).await {
                Ok(hash) => hash,
                Err(e) => {
                    error!("SESSION TOKEN HASHING ERROR!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Session Token Hashing Error".to_string(),
                            response_message: e.to_string(),
                        }),
                    ));
                }
            };

            let renewed_session = match renew_session(
                &state.db,
                session.id,
                new_session_token_hash,
                renewed_expires_at,
            )
            .await
            {
                Ok(Some(session)) => session,

                Ok(None) => {
                    error!("SESSION NOT FOUND DURING RENEWAL!");

                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse {
                            error: "Not Found".to_string(),
                            response_message: "Session not found".to_string(),
                        }),
                    ));
                }

                Err(e) => {
                    error!("SESSION RENEWAL FAILED!");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "DB Error".to_string(),
                            response_message: e.to_string(),
                        }),
                    ));
                }
            };

            // Insert session data
            req.extensions_mut().insert(SessionsMiddlewareOutput {
                user,
                session: renewed_session,
                session_status: "USER SESSION IS ACTIVE".to_string(),
                access_token: new_access_token,
                session_token: new_session_token,
            });
        }

        Err(err) => match err.kind() {
            ErrorKind::ExpiredSignature => {
                error!("SESSION EXPIRED!");

                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Forbidden".to_string(),
                        response_message: "User session expired, please re-authenticate"
                            .to_string(),
                    }),
                ));
            }

            _ => {
                error!("SESSION VERIFICATION FAILED!");

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "Session token is invalid".to_string(),
                    }),
                ));
            }
        },
    }

    Ok(next.run(req).await)
}
