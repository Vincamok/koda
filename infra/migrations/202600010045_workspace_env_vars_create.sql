CREATE TABLE workspace_env_vars (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    org_id       UUID NOT NULL,
    key          TEXT NOT NULL,
    value_enc    TEXT NOT NULL,
    nonce        TEXT NOT NULL,
    is_secret    BOOL NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, key)
);
CREATE INDEX idx_workspace_env_vars_workspace_id ON workspace_env_vars(workspace_id);
ALTER TABLE workspace_env_vars ENABLE ROW LEVEL SECURITY;
