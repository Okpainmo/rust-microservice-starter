use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SubSession {
    pub id: Uuid,
    pub creation_order: i64,
    pub session_id: Uuid,
    pub user_id: i64,
    pub activity_type: String,
    pub activity_description: Option<String>,
    pub ip_address: Option<String>, // user's current IP
    pub user_agent: Option<String>, // user's current browser agent
    pub request_method: String,
    pub request_path: String,
    pub created_at: NaiveDateTime,
}
