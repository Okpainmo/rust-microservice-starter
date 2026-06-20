use crate::core::structs::permission::Permission;
use sqlx::PgPool;

pub async fn find_all_permissions(db: &PgPool) -> Result<Vec<Permission>, sqlx::Error> {
    sqlx::query_as::<_, Permission>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM permissions
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find_permissions_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<Permission>, sqlx::Error> {
    sqlx::query_as::<_, Permission>(
        r#"
        SELECT DISTINCT p.id, p.name, p.description, p.created_at, p.updated_at
        FROM permissions p
        INNER JOIN role_permissions rp ON rp.permission_id = p.id
        INNER JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = $1
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}
