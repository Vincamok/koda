CREATE TABLE workspace_notes (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID        NOT NULL REFERENCES organizations(id),
    user_id         UUID        NOT NULL REFERENCES users(id),
    content         TEXT        NOT NULL DEFAULT '',
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (workspace_id, user_id)
);

CREATE INDEX idx_workspace_notes_workspace_id ON workspace_notes (workspace_id);
CREATE INDEX idx_workspace_notes_user_id      ON workspace_notes (user_id);

ALTER TABLE workspace_notes ENABLE ROW LEVEL SECURITY;

CREATE POLICY workspace_notes_self ON workspace_notes
    USING (user_id = current_setting('app.current_user_id', true)::uuid);
