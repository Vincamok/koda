CREATE TABLE exposure_rules (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id        UUID        NOT NULL REFERENCES workspaces(id),
    plugin_binding_id   UUID        NOT NULL REFERENCES workspace_plugin_bindings(id),
    rule_type           TEXT        NOT NULL CHECK (rule_type IN ('http', 'tcp')),
    path_prefix         TEXT,
    host_port           INT,
    internal_host       TEXT        NOT NULL,
    internal_port       INT         NOT NULL,
    is_active           BOOL        NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_exposure_rules_workspace_id      ON exposure_rules (workspace_id);
CREATE INDEX idx_exposure_rules_plugin_binding_id ON exposure_rules (plugin_binding_id);
CREATE INDEX idx_exposure_rules_is_active         ON exposure_rules (is_active);
