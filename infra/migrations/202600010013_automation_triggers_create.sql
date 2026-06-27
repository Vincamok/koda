CREATE TABLE automation_triggers (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id  UUID        NOT NULL REFERENCES workspaces(id),
    pipeline_id   UUID        NOT NULL REFERENCES cicd_pipelines(id),
    trigger_type  TEXT        NOT NULL CHECK (trigger_type IN ('on_push', 'schedule', 'manual')),
    schedule_cron TEXT,
    is_active     BOOL        NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_automation_triggers_workspace_id ON automation_triggers (workspace_id);
CREATE INDEX idx_automation_triggers_pipeline_id  ON automation_triggers (pipeline_id);
CREATE INDEX idx_automation_triggers_is_active    ON automation_triggers (is_active);
