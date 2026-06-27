CREATE TABLE secret_refs (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID        REFERENCES organizations(id),
    user_id          UUID        REFERENCES users(id),
    workspace_id     UUID        REFERENCES workspaces(id),
    name             TEXT        NOT NULL,
    encrypted_value  BYTEA       NOT NULL,
    nonce            BYTEA       NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT secret_refs_must_have_owner CHECK (
        organization_id IS NOT NULL OR user_id IS NOT NULL
    )
);

CREATE INDEX idx_secret_refs_organization_id ON secret_refs (organization_id);
CREATE INDEX idx_secret_refs_user_id         ON secret_refs (user_id);
CREATE INDEX idx_secret_refs_workspace_id    ON secret_refs (workspace_id);

-- Deferred FK: add the FK from workspace_git_configs now that secret_refs exists
ALTER TABLE workspace_git_configs
    ADD CONSTRAINT fk_workspace_git_ssh_secret
    FOREIGN KEY (ssh_key_secret_ref_id) REFERENCES secret_refs(id);
