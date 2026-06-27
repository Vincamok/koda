CREATE TABLE workspace_snapshots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    created_by  UUID NOT NULL REFERENCES users(id),
    label       TEXT NOT NULL,
    volume_snapshot_path TEXT NOT NULL,
    container_state JSONB NOT NULL DEFAULT '{}',
    size_bytes  BIGINT,
    status      TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'creating', 'ready', 'failed', 'deleted')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspace_snapshots_workspace_id ON workspace_snapshots (workspace_id);
CREATE INDEX idx_workspace_snapshots_organization_id ON workspace_snapshots (organization_id);
