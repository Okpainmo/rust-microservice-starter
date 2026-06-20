use crate::core::structs::session::Session;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_all_sessions(db: &PgPool) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT
            id,
            creation_order,
            user_id,
            refresh_token_hash,
            expires_at,
            status,
            revoked_at,
            created_at,
            updated_at
        FROM sessions
        ORDER BY creation_order ASC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find_session_by_id(
    db: &PgPool,
    session_id: Uuid,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT
            id,
            creation_order,
            user_id,
            refresh_token_hash,
            expires_at,
            status,
            revoked_at,
            created_at,
            updated_at
        FROM sessions
        WHERE id = $1
        "#,
    )
    .bind(session_id)
    .fetch_optional(db)
    .await
}

pub async fn find_active_sessions_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT
            id,
            creation_order,
            user_id,
            refresh_token_hash,
            expires_at,
            status,
            revoked_at,
            created_at,
            updated_at
        FROM sessions
        WHERE user_id = $1
            AND status = 'active'
            AND expires_at > NOW()
        ORDER BY creation_order ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn find_sessions_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT
            id,
            creation_order,
            user_id,
            refresh_token_hash,
            expires_at,
            status,
            revoked_at,
            created_at,
            updated_at
        FROM sessions
        WHERE user_id = $1
        ORDER BY creation_order ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}
