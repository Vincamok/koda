CREATE TABLE refactor_runs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    pipeline_id     UUID REFERENCES cicd_pipelines(id) ON DELETE SET NULL,
    status          TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    summary         TEXT,
    suggestions     JSONB NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refactor_runs_workspace ON refactor_runs(workspace_id, created_at DESC);
ALTER TABLE refactor_runs ENABLE ROW LEVEL SECURITY;
