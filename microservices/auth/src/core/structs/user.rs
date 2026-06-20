use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub profile_image: Option<String>,
    #[serde(skip_serializing)]
    pub password: String,
    pub user_type: String,
    pub is_active: bool,
    pub status: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
    pub is_logged_out: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RegisteredUserProfile {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub profile_image: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
    pub user_type: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserLookUp {
    pub email: String,
    pub phone_number: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
