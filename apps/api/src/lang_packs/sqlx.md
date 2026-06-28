# SQLx Framework Pack

You are working with **SQLx 0.7** (async Rust SQL toolkit, PostgreSQL).

## Query macros

```rust
// Checked at compile time — preferred
let row = sqlx::query!(
    "SELECT id, name, status FROM workspaces WHERE id = $1 AND organization_id = $2",
    workspace_id,
    org_id
)
.fetch_one(&pool)
.await?;

// Dynamic queries — use query_builder or raw query
let mut qb = sqlx::QueryBuilder::new("SELECT id FROM workspaces WHERE 1=1");
if let Some(status) = filter_status {
    qb.push(" AND status = ");
    qb.push_bind(status);
}
let rows = qb.build().fetch_all(&pool).await?;
```

## Fetch variants
- `fetch_one` — exactly 1 row, error if 0 or >1
- `fetch_optional` — 0 or 1 row → `Option<Row>`
- `fetch_all` — all rows → `Vec<Row>`
- `execute` — INSERT/UPDATE/DELETE, returns `PgQueryResult`

## Transactions
```rust
let mut tx = pool.begin().await?;
sqlx::query!("INSERT INTO ...").execute(&mut *tx).await?;
sqlx::query!("UPDATE ...").execute(&mut *tx).await?;
tx.commit().await?;
// tx.rollback() on error (or auto-rollback on drop)
```

## Types
- `Uuid` → `uuid` feature enabled
- `OffsetDateTime` → `time` feature enabled  
- `Vec<T>` → PostgreSQL arrays
- `Option<T>` → nullable columns

## Multi-tenancy invariant
**ALWAYS** filter by `organization_id` on every query touching org-scoped entities:
```rust
"WHERE id = $1 AND organization_id = $2"
//                  ^^^^^^^^^^^^^^^^^^^ never omit this
```

## Migrations
- Files in `infra/migrations/` — named `YYYYMMDDHHMMSS_description.sql`
- `sqlx migrate run --source ../../infra/migrations` to apply
- Never modify applied migrations — create new ones

## Connection pool
```rust
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```
