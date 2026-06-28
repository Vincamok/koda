-- Workspace hibernation config: idle threshold per org (overridable per workspace)
CREATE TABLE workspace_hibernation_configs (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id       UUID        NOT NULL UNIQUE,
    idle_threshold_minutes INT        NOT NULL DEFAULT 30 CHECK (idle_threshold_minutes >= 5),
    enabled               BOOL        NOT NULL DEFAULT true,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Track last activity per workspace (bumped on WebSocket message, AI chat, file edit)
ALTER TABLE workspaces ADD COLUMN IF NOT EXISTS last_activity_at TIMESTAMPTZ;

CREATE INDEX idx_workspaces_last_activity ON workspaces(last_activity_at)
    WHERE status = 'running';

ALTER TABLE workspace_hibernation_configs ENABLE ROW LEVEL SECURITY;
