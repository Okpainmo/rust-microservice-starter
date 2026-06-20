use crate::core::structs::session::Session;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn revoke_session_by_id_and_user_id(
    db: &PgPool,
    session_id: Uuid,
    user_id: i64,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        UPDATE sessions
        SET
            status = 'revoked',
            revoked_at = COALESCE(revoked_at, NOW()),
            updated_at = NOW()
        WHERE id = $1
            AND user_id = $2
            AND status = 'active'
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
    .bind(session_id)
    .bind(user_id)
    .fetch_optional(db)
    .await
}
