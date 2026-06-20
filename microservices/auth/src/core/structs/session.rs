use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct Session {
    pub id: Uuid,
    pub creation_order: i64,
    pub user_id: i64,
    pub refresh_token_hash: String,
    pub expires_at: NaiveDateTime,
    pub status: String,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
