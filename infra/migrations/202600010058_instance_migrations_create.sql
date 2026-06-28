-- Track org migrations between Koda instances
CREATE TABLE instance_org_migrations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    source_instance UUID NOT NULL REFERENCES koda_instances(id),
    target_instance UUID NOT NULL REFERENCES koda_instances(id),
    status          TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'in_progress', 'completed', 'failed', 'rolled_back')),
    initiated_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    -- JSON snapshot of what was migrated and any error details
    progress        JSONB NOT NULL DEFAULT '{}',
    error           TEXT,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_instance_migrations_org ON instance_org_migrations(organization_id, created_at DESC);
CREATE INDEX idx_instance_migrations_status ON instance_org_migrations(status) WHERE status IN ('pending','in_progress');

-- Instance load stats (updated by each instance on health-check interval)
ALTER TABLE koda_instances
    ADD COLUMN IF NOT EXISTS workspace_count  INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS cpu_usage_pct    DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS ram_usage_pct    DOUBLE PRECISION;
