use crate::core::structs::permission::Permission;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UpdatePermission {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn update_permission(
    db: &PgPool,
    permission_id: Uuid,
    permission: UpdatePermission,
) -> Result<Option<Permission>, sqlx::Error> {
    sqlx::query_as::<_, Permission>(
        r#"
        UPDATE permissions
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, created_at, updated_at
        "#,
    )
    .bind(permission_id)
    .bind(permission.name)
    .bind(permission.description)
    .fetch_optional(db)
    .await
}
