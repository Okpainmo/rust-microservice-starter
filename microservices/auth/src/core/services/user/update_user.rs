use sqlx::PgPool;

#[derive(Debug, Default)]
pub struct UpdateUser {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub is_logged_out: Option<bool>,
}

pub async fn update_user_by_email(
    db: &PgPool,
    email: &str,
    user: UpdateUser,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
            UPDATE users
            SET
                access_token = COALESCE($1, access_token),
                refresh_token = COALESCE($2, refresh_token),
                is_logged_out = COALESCE($3, is_logged_out),
                updated_at = NOW()
            WHERE email = $4
        "#,
    )
    .bind(user.access_token)
    .bind(user.refresh_token)
    .bind(user.is_logged_out)
    .bind(email)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub async fn update_user_by_id(
    db: &PgPool,
    user_id: i64,
    user: UpdateUser,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
            UPDATE users
            SET
                access_token = COALESCE($1, access_token),
                refresh_token = COALESCE($2, refresh_token),
                is_logged_out = COALESCE($3, is_logged_out),
                updated_at = NOW()
            WHERE id = $4
        "#,
    )
    .bind(user.access_token)
    .bind(user.refresh_token)
    .bind(user.is_logged_out)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
