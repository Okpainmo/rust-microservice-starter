use crate::core::structs::permission::Permission;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CreatePermission {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_permission(
    db: &PgPool,
    permission: CreatePermission,
) -> Result<Permission, sqlx::Error> {
    sqlx::query_as::<_, Permission>(
        r#"
        INSERT INTO permissions (id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(permission.name)
    .bind(permission.description)
    .fetch_one(db)
    .await
}
