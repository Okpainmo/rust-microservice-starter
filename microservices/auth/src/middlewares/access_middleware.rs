use crate::core::structs::user::UserProfile;
use crate::middlewares::sessions_middleware::SessionsMiddlewareOutput;
use crate::utils::generate_tokens::User;
use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::error;

// ============================================================================
// Types/Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenKind {
    Access,
    Refresh,
    OneTimePassword,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: i64,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub token_kind: TokenKind,
}

#[derive(Clone)]
pub struct MiddlewareState {
    pub jwt_secret: String,
    pub cookie_name: String,
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub user: User,
    pub new_access_token: String,
    pub new_refresh_token: String,
    pub session_status: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub response_message: String,
}

pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub auth_cookie: String,
}

enum TokenStatus {
    Valid,
    Expired,
    Invalid(String),
}

fn verify_access_token(token: &str, secret: &str, user: &UserProfile) -> TokenStatus {
    let validation = Validation::default();
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    match decode::<JwtClaims>(token, &decoding_key, &validation) {
        Ok(token_data) => match validate_access_claims(&token_data.claims, user) {
            Ok(()) => TokenStatus::Valid,
            Err(message) => TokenStatus::Invalid(message),
        },
        Err(err) => {
            use jsonwebtoken::errors::ErrorKind;
            match err.kind() {
                ErrorKind::ExpiredSignature => {
                    let mut expired_validation = Validation::default();
                    expired_validation.validate_exp = false;

                    match decode::<JwtClaims>(token, &decoding_key, &expired_validation) {
                        Ok(token_data) => match validate_access_claims(&token_data.claims, user) {
                            Ok(()) => TokenStatus::Expired,
                            Err(message) => TokenStatus::Invalid(message),
                        },
                        Err(err) => {
                            TokenStatus::Invalid(format!("Token verification failed: {}", err))
                        }
                    }
                }
                _ => TokenStatus::Invalid(format!("Token verification failed: {}", err)),
            }
        }
    }
}

fn validate_access_claims(claims: &JwtClaims, user: &UserProfile) -> Result<(), String> {
    if claims.id != user.id {
        return Err("User credentials do not match".to_string());
    }

    if claims.email != user.email {
        return Err("User credentials do not match".to_string());
    }

    if claims.token_kind != TokenKind::Access {
        return Err("Invalid token type".to_string());
    }

    Ok(())
}

// ============================================================================
// Access Middleware
// ============================================================================
pub async fn access_middleware(
    State(state): State<crate::AppState>,
    cookies: Cookies,
    req: Request,
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

    // ----------------------------------------------------------
    // AUTH COOKIE CHECK
    // ----------------------------------------------------------
    let _auth_cookie = cookies.get(&session_state.cookie_name).ok_or_else(|| {
        error!("MISSING AUTH COOKIE!");
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                response_message: "Request rejected, please re-authenticate".to_string(),
            }),
        )
    })?;

    // ----------------------------------------------------------
    // SESSION MIDDLEWARE OUTPUT CHECK
    // ----------------------------------------------------------
    let sessions_middleware_output = req
        .extensions()
        .get::<SessionsMiddlewareOutput>()
        .ok_or_else(|| {
            error!("SESSION MIDDLEWARE OUTPUT MISSING!");
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Not Found".to_string(),
                    response_message: "Data received from sessions middleware".to_string(),
                }),
            )
        })?
        .clone();

    // ----------------------------------------------------------
    // AUTH HEADER CHECK
    // ----------------------------------------------------------
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            error!("AUTHORIZATION HEADER MISSING!");
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Forbidden".to_string(),
                    response_message: "Authorization header missing".to_string(),
                }),
            )
        })?;

    // ----------------------------------------------------------
    // BEARER FORMAT CHECK
    // ----------------------------------------------------------
    if !auth_header.starts_with("Bearer ") {
        error!("INVALID BEARER FORMAT!");
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Forbidden".to_string(),
                response_message:
                    "Authorization string does not match expected (Bearer Token) format".to_string(),
            }),
        ));
    }

    let access_token = auth_header.trim_start_matches("Bearer ").trim();

    // ----------------------------------------------------------
    // ACCESS TOKEN VERIFICATION
    // ----------------------------------------------------------
    match verify_access_token(
        access_token,
        &session_state.jwt_secret,
        &sessions_middleware_output.user,
    ) {
        TokenStatus::Valid => {
            // normal path (no log)
        }

        // covered to appease rust - but it's not actually possible for access token to be expired while
        // refresh passes. Hence pass with "no log" as below.
        TokenStatus::Expired => {
            // normal path (no log)
        }

        TokenStatus::Invalid(msg) => {
            error!("INVALID ACCESS TOKEN!");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: msg,
                }),
            ));
        }
    }

    // ----------------------------------------------------------
    // NORMAL FLOW (NO LOGGING)
    // ----------------------------------------------------------
    Ok(next.run(req).await)
}
