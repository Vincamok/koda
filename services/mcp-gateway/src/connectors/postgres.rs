use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use super::{McpConnector, McpResult};

pub struct PostgresConnector;

#[async_trait]
impl McpConnector for PostgresConnector {
    fn id(&self) -> &'static str { "postgres" }

    fn list_tools(&self) -> Vec<&'static str> {
        vec!["postgres_query", "postgres_list_tables", "postgres_describe_table"]
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult> {
        let conn_str  = config["connection_string"].as_str().ok_or_else(|| anyhow::anyhow!("connection_string manquant"))?;
        let readonly  = config.get("readonly").and_then(|v| v.as_bool()).unwrap_or(true);
        let max_rows  = config.get("max_rows").and_then(|v| v.as_u64()).unwrap_or(100);

        let pool = sqlx::PgPool::connect(conn_str).await?;

        let result = match tool_name {
            "postgres_query" => {
                let sql = arguments["sql"].as_str().ok_or_else(|| anyhow::anyhow!("sql manquant"))?;

                if readonly && is_write_query(sql) {
                    anyhow::bail!("Requête d'écriture interdite en mode readonly");
                }

                let limited_sql = format!("SELECT * FROM ({sql}) __koda_q LIMIT {max_rows}");
                let rows = sqlx::query(&limited_sql).fetch_all(&pool).await?;

                let json_rows: Vec<Value> = rows.iter().map(row_to_json).collect();
                serde_json::json!({ "rows": json_rows, "count": json_rows.len() })
            }
            "postgres_list_tables" => {
                let schema = arguments.get("schema").and_then(|v| v.as_str()).unwrap_or("public");
                let rows = sqlx::query(
                    "SELECT table_name, table_type FROM information_schema.tables WHERE table_schema = $1 ORDER BY table_name"
                ).bind(schema).fetch_all(&pool).await?;
                serde_json::json!({ "tables": rows.iter().map(row_to_json).collect::<Vec<_>>() })
            }
            "postgres_describe_table" => {
                let table  = arguments["table"].as_str().ok_or_else(|| anyhow::anyhow!("table manquant"))?;
                let schema = arguments.get("schema").and_then(|v| v.as_str()).unwrap_or("public");
                let rows = sqlx::query(
                    "SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position"
                ).bind(schema).bind(table).fetch_all(&pool).await?;
                serde_json::json!({ "columns": rows.iter().map(row_to_json).collect::<Vec<_>>() })
            }
            _ => anyhow::bail!("Outil inconnu : {tool_name}"),
        };

        pool.close().await;
        Ok(McpResult { content: result, is_error: false })
    }

    async fn read_resource(&self, _uri: &str, _config: &HashMap<String, Value>) -> anyhow::Result<McpResult> {
        anyhow::bail!("read_resource non supporté pour PostgreSQL")
    }
}

fn is_write_query(sql: &str) -> bool {
    let upper = sql.trim().to_uppercase();
    ["INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER", "TRUNCATE"]
        .iter()
        .any(|kw| upper.starts_with(kw))
}

fn row_to_json(row: &sqlx::postgres::PgRow) -> Value {
    use sqlx::Row;
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let val: Value = row.try_get(col.name()).unwrap_or(Value::Null);
        map.insert(col.name().to_string(), val);
    }
    Value::Object(map)
}
