use crate::core::structs::sub_session::SubSession;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_sub_sessions_by_session_id(
    db: &PgPool,
    session_id: Uuid,
) -> Result<Vec<SubSession>, sqlx::Error> {
    sqlx::query_as::<_, SubSession>(
        r#"
        SELECT
            id,
            creation_order,
            session_id,
            user_id,
            activity_type,
            activity_description,
            ip_address,
            user_agent,
            request_method,
            request_path,
            created_at
        FROM sub_sessions
        WHERE session_id = $1
        ORDER BY creation_order ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(db)
    .await
}
