use crate::core::structs::role::Role;
use sqlx::PgPool;

pub async fn find_all_roles(db: &PgPool) -> Result<Vec<Role>, sqlx::Error> {
    sqlx::query_as::<_, Role>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM roles
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}
