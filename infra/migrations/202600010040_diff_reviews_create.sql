CREATE TABLE diff_reviews (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id     UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id  UUID        NOT NULL REFERENCES organizations(id),
    pipeline_id      UUID        REFERENCES cicd_pipelines(id),
    status           TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    summary          TEXT,
    review_text      TEXT,
    files_changed    INT,
    insertions       INT,
    deletions        INT,
    base_ref         TEXT,
    head_ref         TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_diff_reviews_workspace_id    ON diff_reviews (workspace_id);
CREATE INDEX idx_diff_reviews_organization_id ON diff_reviews (organization_id);
CREATE INDEX idx_diff_reviews_pipeline_id     ON diff_reviews (pipeline_id);
CREATE INDEX idx_diff_reviews_status          ON diff_reviews (status);

ALTER TABLE diff_reviews ENABLE ROW LEVEL SECURITY;
