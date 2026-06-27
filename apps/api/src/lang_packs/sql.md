## SQL — Koda conventions

- PostgreSQL 16+
- Always filter by `organization_id` on tenant tables — no exceptions
- Never `SELECT *` on business tables — always name columns explicitly
- Migrations: sqlx-migrate, naming `YYYYMMDDHHMM_<object>_<action>.sql`
- NOT NULL columns added in 3 steps: nullable → backfill → NOT NULL constraint
- Indexes: always on FKs, status columns, and frequently filtered columns
- Never DROP a column without 2-week deprecation delay
- Transactions: wrap multi-table writes; use `FOR UPDATE SKIP LOCKED` for job queues
- UUIDs: `gen_random_uuid()` as default, never sequential integers for business entities
