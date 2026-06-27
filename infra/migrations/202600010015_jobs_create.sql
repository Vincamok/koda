CREATE TABLE jobs (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type    TEXT        NOT NULL,
    payload     JSONB       NOT NULL DEFAULT '{}',
    status      TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'success', 'failed')),
    result      JSONB,
    error       TEXT,
    attempts    INT         NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_jobs_status   ON jobs (status);
CREATE INDEX idx_jobs_job_type ON jobs (job_type);
