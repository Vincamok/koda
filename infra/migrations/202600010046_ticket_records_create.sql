CREATE TABLE ticket_records (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id),
    organization_id UUID NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT,
    status          TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'closed')),
    priority        TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('critical', 'high', 'medium', 'low')),
    external_url    TEXT,
    external_system TEXT CHECK (external_system IN ('jira', 'linear', 'github', 'gitlab', 'notion')),
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_ticket_records_workspace_id ON ticket_records(workspace_id);
CREATE INDEX idx_ticket_records_org_id ON ticket_records(organization_id);
ALTER TABLE ticket_records ENABLE ROW LEVEL SECURITY;
