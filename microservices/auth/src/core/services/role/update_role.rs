use crate::core::structs::role::Role;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn update_role(
    db: &PgPool,
    role_id: Uuid,
    role: UpdateRole,
) -> Result<Option<Role>, sqlx::Error> {
    sqlx::query_as::<_, Role>(
        r#"
        UPDATE roles
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, created_at, updated_at
        "#,
    )
    .bind(role_id)
    .bind(role.name)
    .bind(role.description)
    .fetch_optional(db)
    .await
}
