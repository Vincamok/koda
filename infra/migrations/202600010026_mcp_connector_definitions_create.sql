CREATE TABLE mcp_connector_definitions (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    slug           TEXT        NOT NULL UNIQUE,
    name           TEXT        NOT NULL,
    description    TEXT,
    version        TEXT        NOT NULL,
    category       TEXT        NOT NULL,
    capabilities   JSONB       NOT NULL DEFAULT '[]',
    config_fields  JSONB       NOT NULL DEFAULT '[]',
    tools          JSONB       NOT NULL DEFAULT '[]',
    is_builtin     BOOL        NOT NULL DEFAULT false,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_mcp_connector_definitions_slug       ON mcp_connector_definitions (slug);
CREATE INDEX idx_mcp_connector_definitions_category   ON mcp_connector_definitions (category);
CREATE INDEX idx_mcp_connector_definitions_is_builtin ON mcp_connector_definitions (is_builtin);
