use sqlx::PgPool;
use uuid::Uuid;

pub async fn record_audit_event(
    pool: &PgPool,
    actor_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<&str>,
    metadata: serde_json::Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"INSERT INTO audit_events
               (actor_id, organization_id, action, resource_type, resource_id, metadata, ip_address, user_agent)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        actor_id,
        organization_id,
        action,
        resource_type,
        resource_id,
        metadata,
        ip_address,
        user_agent,
    )
    .execute(pool)
    .await?;
    Ok(())
}
