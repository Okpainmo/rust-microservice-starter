use sqlx::PgPool;
use uuid::Uuid;

pub async fn assign_role_permission(
    db: &PgPool,
    role_id: Uuid,
    permission_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO role_permissions (role_id, permission_id)
        VALUES ($1, $2)
        ON CONFLICT (role_id, permission_id) DO NOTHING
        "#,
    )
    .bind(role_id)
    .bind(permission_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_role_permission(
    db: &PgPool,
    role_id: Uuid,
    permission_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM role_permissions
        WHERE role_id = $1
            AND permission_id = $2
        "#,
    )
    .bind(role_id)
    .bind(permission_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
