-- Note: ssh_key_secret_ref_id FK to secret_refs is added in migration 202600010017_secret_refs_create.sql
-- once the secret_refs table exists, to avoid a circular dependency.
CREATE TABLE workspace_git_configs (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id          UUID        NOT NULL UNIQUE REFERENCES workspaces(id),
    repo_url              TEXT        NOT NULL,
    branch                TEXT        NOT NULL DEFAULT 'main',
    clone_status          TEXT        NOT NULL DEFAULT 'pending' CHECK (clone_status IN ('pending', 'cloning', 'ready', 'failed')),
    clone_error           TEXT,
    ssh_key_secret_ref_id UUID,
    last_cloned_at        TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspace_git_configs_workspace_id ON workspace_git_configs (workspace_id);
