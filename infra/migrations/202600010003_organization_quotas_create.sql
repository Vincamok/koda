CREATE TABLE organization_quotas (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID        NOT NULL UNIQUE REFERENCES organizations(id),
    max_workspaces   INT         NOT NULL DEFAULT 10,
    max_cpu_cores    INT         NOT NULL DEFAULT 20,
    max_ram_gb       INT         NOT NULL DEFAULT 40,
    max_storage_gb   INT         NOT NULL DEFAULT 200,
    max_members      INT         NOT NULL DEFAULT 50,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_organization_quotas_organization_id ON organization_quotas (organization_id);
