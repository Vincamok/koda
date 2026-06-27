CREATE TABLE team_project_access (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id     UUID        NOT NULL REFERENCES teams(id),
    project_id  UUID        NOT NULL REFERENCES projects(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, project_id)
);

CREATE INDEX idx_team_project_access_team_id    ON team_project_access (team_id);
CREATE INDEX idx_team_project_access_project_id ON team_project_access (project_id);
