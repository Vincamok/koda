CREATE TABLE scan_rules (
    id                 UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id    UUID        REFERENCES organizations(id),
    workspace_id       UUID        REFERENCES workspaces(id),
    name               TEXT        NOT NULL,
    rule_type          TEXT        NOT NULL CHECK (rule_type IN ('regex', 'entropy', 'composite')),
    pattern            TEXT,
    entropy_threshold  FLOAT,
    severity           TEXT        NOT NULL DEFAULT 'high' CHECK (severity IN ('critical', 'high', 'medium', 'low')),
    is_builtin         BOOL        NOT NULL DEFAULT false,
    is_active          BOOL        NOT NULL DEFAULT true,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT scan_rules_builtin_check CHECK (
        is_builtin = false OR (organization_id IS NULL AND workspace_id IS NULL)
    )
);

CREATE INDEX idx_scan_rules_organization_id ON scan_rules (organization_id);
CREATE INDEX idx_scan_rules_workspace_id    ON scan_rules (workspace_id);
CREATE INDEX idx_scan_rules_is_builtin      ON scan_rules (is_builtin);
CREATE INDEX idx_scan_rules_is_active       ON scan_rules (is_active);
