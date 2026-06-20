use crate::core::structs::session::Session;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CreateSession {
    pub user_id: i64,
    pub refresh_token_hash: String,
    pub expires_at: NaiveDateTime,
}

pub async fn create_session(db: &PgPool, session: CreateSession) -> Result<Session, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (
            id,
            user_id,
            refresh_token_hash,
            expires_at
        )
        VALUES ($1, $2, $3, $4)
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
    .bind(Uuid::new_v4())
    .bind(session.user_id)
    .bind(session.refresh_token_hash)
    .bind(session.expires_at)
    .fetch_one(db)
    .await
}
