CREATE TABLE memberships (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID        NOT NULL REFERENCES organizations(id),
    user_id          UUID        NOT NULL REFERENCES users(id),
    role             TEXT        NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    invited_by       UUID        REFERENCES users(id),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, user_id)
);

CREATE INDEX idx_memberships_organization_id ON memberships (organization_id);
CREATE INDEX idx_memberships_user_id         ON memberships (user_id);
