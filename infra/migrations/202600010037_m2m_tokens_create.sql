CREATE TABLE m2m_tokens (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID      NOT NULL REFERENCES organizations(id),
    created_by    UUID        NOT NULL REFERENCES users(id),
    name          TEXT        NOT NULL,
    token_hash    TEXT        NOT NULL UNIQUE,
    token_prefix  TEXT        NOT NULL,
    scopes        TEXT[]      NOT NULL DEFAULT '{}',
    last_used_at  TIMESTAMPTZ,
    expires_at    TIMESTAMPTZ,
    revoked_at    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_m2m_tokens_organization_id ON m2m_tokens (organization_id);
CREATE INDEX idx_m2m_tokens_token_hash      ON m2m_tokens (token_hash);
CREATE INDEX idx_m2m_tokens_created_by      ON m2m_tokens (created_by);
