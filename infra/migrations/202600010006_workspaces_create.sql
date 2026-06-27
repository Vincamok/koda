CREATE TABLE workspaces (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    uid              TEXT        NOT NULL UNIQUE,
    organization_id  UUID        NOT NULL REFERENCES organizations(id),
    project_id       UUID        REFERENCES projects(id),
    template_id      UUID        REFERENCES templates(id),
    created_by       UUID        REFERENCES users(id),
    name             TEXT        NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'created' CHECK (status IN (
                         'created', 'cloning', 'ready', 'starting', 'running',
                         'stopping', 'stopped', 'reviewing', 'closed', 'failed'
                     )),
    cpu_limit        INT         NOT NULL DEFAULT 2,
    ram_limit_mb     INT         NOT NULL DEFAULT 2048,
    pids_limit       INT         NOT NULL DEFAULT 512,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspaces_organization_id ON workspaces (organization_id);
CREATE INDEX idx_workspaces_project_id      ON workspaces (project_id);
CREATE INDEX idx_workspaces_status          ON workspaces (status);
CREATE INDEX idx_workspaces_created_by      ON workspaces (created_by);
CREATE INDEX idx_workspaces_uid             ON workspaces (uid);
