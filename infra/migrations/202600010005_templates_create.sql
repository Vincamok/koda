CREATE TABLE templates (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id   UUID        NOT NULL REFERENCES organizations(id),
    name              TEXT        NOT NULL,
    description       TEXT,
    docker_image      TEXT        NOT NULL,
    cpu_limit         INT         NOT NULL DEFAULT 2,
    ram_limit_mb      INT         NOT NULL DEFAULT 2048,
    plugin_ids        JSONB       NOT NULL DEFAULT '[]',
    devcontainer_json JSONB,
    is_public         BOOL        NOT NULL DEFAULT false,
    created_by        UUID        REFERENCES users(id),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_templates_organization_id ON templates (organization_id);
CREATE INDEX idx_templates_created_by      ON templates (created_by);
CREATE INDEX idx_templates_is_public       ON templates (is_public);
