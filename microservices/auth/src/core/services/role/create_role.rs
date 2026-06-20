use crate::core::structs::role::Role;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CreateRole {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_role(db: &PgPool, role: CreateRole) -> Result<Role, sqlx::Error> {
    sqlx::query_as::<_, Role>(
        r#"
        INSERT INTO roles (id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(role.name)
    .bind(role.description)
    .fetch_one(db)
    .await
}
