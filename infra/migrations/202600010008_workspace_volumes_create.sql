CREATE TABLE workspace_volumes (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id  UUID        NOT NULL REFERENCES workspaces(id),
    volume_name   TEXT        NOT NULL UNIQUE,
    size_gb       INT         NOT NULL DEFAULT 10,
    status        TEXT        NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'detached', 'archived', 'deleted')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspace_volumes_workspace_id ON workspace_volumes (workspace_id);
CREATE INDEX idx_workspace_volumes_status       ON workspace_volumes (status);
