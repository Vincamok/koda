CREATE TABLE workspace_mcp_bindings (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id            UUID        NOT NULL REFERENCES workspaces(id),
    connector_definition_id UUID        NOT NULL REFERENCES mcp_connector_definitions(id),
    config                  JSONB       NOT NULL DEFAULT '{}',
    enabled                 BOOL        NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (workspace_id, connector_definition_id)
);

CREATE INDEX idx_workspace_mcp_bindings_workspace_id            ON workspace_mcp_bindings (workspace_id);
CREATE INDEX idx_workspace_mcp_bindings_connector_definition_id ON workspace_mcp_bindings (connector_definition_id);
