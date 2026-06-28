CREATE TABLE alert_rules (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID        NOT NULL,
    name            TEXT        NOT NULL,
    rule_type       TEXT        NOT NULL CHECK (rule_type IN (
                        'crash_loop', 'memory_exceeded', 'cpu_exceeded',
                        'pipeline_failed', 'workspace_stuck', 'quota_near_limit'
                    )),
    threshold       NUMERIC,           -- context-dependent: restart count, %, etc.
    webhook_url     TEXT,              -- Slack / Discord / custom webhook
    enabled         BOOL        NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE alert_events (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id         UUID        NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    organization_id UUID        NOT NULL,
    workspace_id    UUID        REFERENCES workspaces(id),
    severity        TEXT        NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low', 'info')),
    message         TEXT        NOT NULL,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    notified_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alert_rules_org ON alert_rules(organization_id);
CREATE INDEX idx_alert_events_org ON alert_events(organization_id);
CREATE INDEX idx_alert_events_workspace ON alert_events(workspace_id);
CREATE INDEX idx_alert_events_created ON alert_events(created_at DESC);

ALTER TABLE alert_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE alert_events ENABLE ROW LEVEL SECURITY;
