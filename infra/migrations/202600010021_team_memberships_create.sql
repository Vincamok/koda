CREATE TABLE team_memberships (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id     UUID        NOT NULL REFERENCES teams(id),
    user_id     UUID        NOT NULL REFERENCES users(id),
    role        TEXT        NOT NULL CHECK (role IN ('lead', 'developer', 'reviewer', 'viewer')),
    granted_by  UUID        REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, user_id)
);

CREATE INDEX idx_team_memberships_team_id ON team_memberships (team_id);
CREATE INDEX idx_team_memberships_user_id ON team_memberships (user_id);
