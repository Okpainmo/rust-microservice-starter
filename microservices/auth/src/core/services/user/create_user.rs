use crate::core::structs::user::RegisteredUserProfile;
use sqlx::PgPool;

pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub profile_image: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
    pub user_type: String,
}

pub async fn create_user(
    db: &PgPool,
    user: CreateUser,
) -> Result<RegisteredUserProfile, sqlx::Error> {
    sqlx::query_as::<_, RegisteredUserProfile>(
        r#"
        INSERT INTO users (
            email,
            password,
            full_name,
            profile_image,
            country,
            country_code,
            phone_number,
            user_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            full_name,
            email,
            profile_image,
            country,
            country_code,
            phone_number,
            user_type,
            created_at,
            updated_at
        "#,
    )
    .bind(user.email)
    .bind(user.password)
    .bind(user.full_name)
    .bind(user.profile_image)
    .bind(user.country)
    .bind(user.country_code)
    .bind(user.phone_number)
    .bind(user.user_type)
    .fetch_one(db)
    .await
}
