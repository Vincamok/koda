CREATE TABLE shadow_deploys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    pipeline_id     UUID REFERENCES cicd_pipelines(id) ON DELETE SET NULL,
    status          TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    primary_output  TEXT,
    shadow_output   TEXT,
    diverged        BOOL,
    error           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_shadow_deploys_workspace ON shadow_deploys(workspace_id, created_at DESC);
ALTER TABLE shadow_deploys ENABLE ROW LEVEL SECURITY;
