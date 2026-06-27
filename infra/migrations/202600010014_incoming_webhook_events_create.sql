CREATE TABLE incoming_webhook_events (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id  UUID        NOT NULL REFERENCES workspaces(id),
    token         TEXT        NOT NULL,
    headers       JSONB       NOT NULL DEFAULT '{}',
    body          JSONB       NOT NULL DEFAULT '{}',
    source_ip     TEXT,
    received_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hmac_valid    BOOL        NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_incoming_webhook_events_workspace_id ON incoming_webhook_events (workspace_id);
CREATE INDEX idx_incoming_webhook_events_received_at  ON incoming_webhook_events (received_at);
