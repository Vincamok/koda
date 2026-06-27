CREATE TABLE workspace_shares (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id          UUID        NOT NULL REFERENCES workspaces(id),
    shared_with_user_id   UUID        REFERENCES users(id),
    shared_with_email     TEXT,
    role                  TEXT        NOT NULL CHECK (role IN ('editor', 'reviewer', 'viewer')),
    expires_at            TIMESTAMPTZ,
    created_by            UUID        REFERENCES users(id),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspace_shares_workspace_id        ON workspace_shares (workspace_id);
CREATE INDEX idx_workspace_shares_shared_with_user_id ON workspace_shares (shared_with_user_id);
