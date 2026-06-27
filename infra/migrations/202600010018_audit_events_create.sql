-- audit_events is immutable: no updated_at column
CREATE TABLE audit_events (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id       UUID        REFERENCES users(id),
    organization_id UUID       REFERENCES organizations(id),
    action         TEXT        NOT NULL,
    resource_type  TEXT,
    resource_id    TEXT,
    metadata       JSONB       NOT NULL DEFAULT '{}',
    ip_address     TEXT,
    user_agent     TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_events_actor_id       ON audit_events (actor_id);
CREATE INDEX idx_audit_events_organization_id ON audit_events (organization_id);
CREATE INDEX idx_audit_events_action          ON audit_events (action);
CREATE INDEX idx_audit_events_resource_type   ON audit_events (resource_type);
CREATE INDEX idx_audit_events_created_at      ON audit_events (created_at);
