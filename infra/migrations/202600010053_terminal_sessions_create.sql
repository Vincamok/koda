CREATE TABLE terminal_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    container_id    TEXT NOT NULL,
    -- 'active' = exec running, 'closed' = session ended
    status          TEXT NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'closed')),
    -- Redis pub/sub channel name for multiplexing output
    redis_channel   TEXT NOT NULL,
    allow_write     BOOL NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at       TIMESTAMPTZ
);

CREATE INDEX idx_terminal_sessions_workspace ON terminal_sessions(workspace_id) WHERE status = 'active';
ALTER TABLE terminal_sessions ENABLE ROW LEVEL SECURITY;
