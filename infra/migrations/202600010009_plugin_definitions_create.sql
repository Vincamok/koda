CREATE TABLE plugin_definitions (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    slug              TEXT        NOT NULL UNIQUE,
    name              TEXT        NOT NULL,
    description       TEXT,
    version           TEXT        NOT NULL,
    plugin_type       TEXT        NOT NULL CHECK (plugin_type IN ('web', 'tcp', 'background')),
    docker_image      TEXT        NOT NULL,
    internal_port     INT,
    health_check_path TEXT,
    network_policy    JSONB       NOT NULL DEFAULT '{}',
    is_builtin        BOOL        NOT NULL DEFAULT false,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plugin_definitions_slug       ON plugin_definitions (slug);
CREATE INDEX idx_plugin_definitions_plugin_type ON plugin_definitions (plugin_type);
CREATE INDEX idx_plugin_definitions_is_builtin  ON plugin_definitions (is_builtin);
