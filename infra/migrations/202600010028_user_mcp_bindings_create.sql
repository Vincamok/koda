CREATE TABLE user_mcp_bindings (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID        NOT NULL REFERENCES users(id),
    connector_definition_id UUID        NOT NULL REFERENCES mcp_connector_definitions(id),
    config                  JSONB       NOT NULL DEFAULT '{}',
    enabled                 BOOL        NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, connector_definition_id)
);

CREATE INDEX idx_user_mcp_bindings_user_id                 ON user_mcp_bindings (user_id);
CREATE INDEX idx_user_mcp_bindings_connector_definition_id ON user_mcp_bindings (connector_definition_id);
