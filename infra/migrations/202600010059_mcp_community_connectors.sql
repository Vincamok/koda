-- Support community (stdio) MCP connectors in the marketplace
ALTER TABLE mcp_connector_definitions
    ADD COLUMN IF NOT EXISTS connector_type TEXT NOT NULL DEFAULT 'builtin'
        CHECK (connector_type IN ('builtin', 'stdio', 'http_sse')),
    ADD COLUMN IF NOT EXISTS community       BOOL NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS author         TEXT,
    ADD COLUMN IF NOT EXISTS repo_url       TEXT,
    ADD COLUMN IF NOT EXISTS approved       BOOL NOT NULL DEFAULT true,
    -- For stdio connectors: the default command template (overridable per binding)
    ADD COLUMN IF NOT EXISTS default_command JSONB;

CREATE INDEX IF NOT EXISTS idx_mcp_connectors_community ON mcp_connector_definitions(community) WHERE community = true;

-- Community stdio connector examples
INSERT INTO mcp_connector_definitions
    (slug, name, description, connector_type, community, approved, default_command, config_schema)
VALUES
    ('filesystem', 'Filesystem', 'Read/write local filesystem via MCP',
     'stdio', true, true,
     '["npx", "-y", "@modelcontextprotocol/server-filesystem", "/workspace"]',
     '{"type":"object","properties":{"root":{"type":"string","description":"Root directory to expose"}}}'),
    ('brave-search', 'Brave Search', 'Web search via Brave Search API',
     'stdio', true, true,
     '["npx", "-y", "@modelcontextprotocol/server-brave-search"]',
     '{"type":"object","properties":{"api_key":{"type":"string","description":"Brave Search API key"}}}')
ON CONFLICT (slug) DO NOTHING;
