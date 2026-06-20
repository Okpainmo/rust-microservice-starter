use crate::core::structs::session::Session;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn update_session_expires_at(
    db: &PgPool,
    session_id: Uuid,
    expires_at: NaiveDateTime,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        UPDATE sessions
        SET
            expires_at = $1,
            updated_at = NOW()
        WHERE id = $2
        RETURNING
            id,
            creation_order,
            user_id,
            refresh_token_hash,
            expires_at,
            status,
            revoked_at,
            created_at,
            updated_at
        "#,
    )
    .bind(expires_at)
    .bind(session_id)
    .fetch_optional(db)
    .await
}

pub async fn renew_session(
    db: &PgPool,
    session_id: Uuid,
    refresh_token_hash: String,
    expires_at: NaiveDateTime,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        UPDATE sessions
        SET
            refresh_token_hash = $1,
            expires_at = $2,
            updated_at = NOW()
        WHERE id = $3
        RETURNING
            id,
            creation_order,
            user_id,
            refresh_token_hash,
            expires_at,
            status,
            revoked_at,
            created_at,
            updated_at
        "#,
    )
    .bind(refresh_token_hash)
    .bind(expires_at)
    .bind(session_id)
    .fetch_optional(db)
    .await
}
