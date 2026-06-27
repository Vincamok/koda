CREATE TABLE security_reports (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id     UUID        NOT NULL REFERENCES workspaces(id),
    organization_id  UUID        NOT NULL REFERENCES organizations(id),
    pipeline_id      UUID        REFERENCES cicd_pipelines(id),
    scan_type        TEXT        NOT NULL CHECK (scan_type IN ('secret_scan', 'sast', 'dependency_scan', 'image_scan')),
    status           TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    summary          TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_reports_workspace_id    ON security_reports (workspace_id);
CREATE INDEX idx_security_reports_organization_id ON security_reports (organization_id);
CREATE INDEX idx_security_reports_pipeline_id     ON security_reports (pipeline_id);
CREATE INDEX idx_security_reports_status          ON security_reports (status);
CREATE INDEX idx_security_reports_scan_type       ON security_reports (scan_type);
