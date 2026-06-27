CREATE TABLE team_quotas (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id          UUID        NOT NULL UNIQUE REFERENCES teams(id),
    max_workspaces   INT         NOT NULL DEFAULT 5,
    max_cpu_cores    INT         NOT NULL DEFAULT 8,
    max_ram_gb       INT         NOT NULL DEFAULT 16,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_team_quotas_team_id ON team_quotas (team_id);
