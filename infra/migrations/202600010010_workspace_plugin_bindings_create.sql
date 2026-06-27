CREATE TABLE workspace_plugin_bindings (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    uid                   TEXT        NOT NULL UNIQUE,
    workspace_id          UUID        NOT NULL REFERENCES workspaces(id),
    plugin_definition_id  UUID        NOT NULL REFERENCES plugin_definitions(id),
    container_id          TEXT,
    status                TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'starting', 'running', 'unhealthy', 'stopped', 'failed')),
    config                JSONB       NOT NULL DEFAULT '{}',
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspace_plugin_bindings_workspace_id         ON workspace_plugin_bindings (workspace_id);
CREATE INDEX idx_workspace_plugin_bindings_plugin_definition_id ON workspace_plugin_bindings (plugin_definition_id);
CREATE INDEX idx_workspace_plugin_bindings_status               ON workspace_plugin_bindings (status);
