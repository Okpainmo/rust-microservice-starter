use crate::core::structs::role::Role;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn assign_user_role_by_name(
    db: &PgPool,
    user_id: i64,
    role_name: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id)
        SELECT $1, id
        FROM roles
        WHERE name = $2
        ON CONFLICT (user_id, role_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(role_name)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub async fn assign_user_role(
    db: &PgPool,
    user_id: i64,
    role_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, role_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(role_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub async fn remove_user_role(
    db: &PgPool,
    user_id: i64,
    role_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM user_roles
        WHERE user_id = $1 AND role_id = $2
        "#,
    )
    .bind(user_id)
    .bind(role_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub async fn find_roles_by_user_id(db: &PgPool, user_id: i64) -> Result<Vec<Role>, sqlx::Error> {
    sqlx::query_as::<_, Role>(
        r#"
        SELECT r.id, r.name, r.description, r.created_at, r.updated_at
        FROM roles r
        INNER JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}
