-- S3-compatible artifact export configuration per org
CREATE TABLE s3_export_configs (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID        NOT NULL UNIQUE,
    endpoint        TEXT        NOT NULL,   -- e.g. https://s3.amazonaws.com or MinIO URL
    bucket          TEXT        NOT NULL,
    region          TEXT        NOT NULL DEFAULT 'us-east-1',
    access_key_enc  TEXT        NOT NULL,   -- AES-256-GCM encrypted
    secret_key_enc  TEXT        NOT NULL,   -- AES-256-GCM encrypted
    path_prefix     TEXT        NOT NULL DEFAULT 'koda-artifacts',
    enabled         BOOL        NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Track exports
CREATE TABLE artifact_exports (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID        NOT NULL,
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id),
    pipeline_run_id UUID,
    artifact_type   TEXT        NOT NULL CHECK (artifact_type IN (
                        'security_report', 'diff_review', 'build_log',
                        'sast_report', 'dependency_report', 'workspace_snapshot'
                    )),
    s3_key          TEXT        NOT NULL,
    s3_url          TEXT,
    size_bytes      BIGINT,
    status          TEXT        NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'uploading', 'completed', 'failed')),
    error           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_artifact_exports_org ON artifact_exports(organization_id);
CREATE INDEX idx_artifact_exports_workspace ON artifact_exports(workspace_id);

ALTER TABLE s3_export_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE artifact_exports ENABLE ROW LEVEL SECURITY;
