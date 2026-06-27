CREATE TABLE cicd_pipelines (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id),
    organization_id UUID        NOT NULL REFERENCES organizations(id),
    name            TEXT        NOT NULL,
    pipeline_type   TEXT        NOT NULL CHECK (pipeline_type IN ('build', 'lint', 'secret_scan', 'sast', 'dependency_scan', 'image_scan')),
    status          TEXT        NOT NULL DEFAULT 'idle' CHECK (status IN ('idle', 'running', 'success', 'failed', 'cancelled')),
    config          JSONB       NOT NULL DEFAULT '{}',
    last_run_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cicd_pipelines_workspace_id    ON cicd_pipelines (workspace_id);
CREATE INDEX idx_cicd_pipelines_organization_id ON cicd_pipelines (organization_id);
CREATE INDEX idx_cicd_pipelines_status          ON cicd_pipelines (status);
CREATE INDEX idx_cicd_pipelines_pipeline_type   ON cicd_pipelines (pipeline_type);
