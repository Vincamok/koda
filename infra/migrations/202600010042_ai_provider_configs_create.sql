CREATE TABLE ai_provider_configs (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id     UUID        UNIQUE REFERENCES organizations(id) ON DELETE CASCADE,
    provider            TEXT        NOT NULL DEFAULT 'anthropic' CHECK (provider IN ('anthropic', 'openai', 'mistral', 'local')),
    model_nano          TEXT        NOT NULL DEFAULT 'claude-haiku-4-5-20251001',
    model_quick         TEXT        NOT NULL DEFAULT 'claude-haiku-4-5-20251001',
    model_standard      TEXT        NOT NULL DEFAULT 'claude-sonnet-4-6',
    model_deep          TEXT        NOT NULL DEFAULT 'claude-sonnet-4-6',
    model_agent         TEXT        NOT NULL DEFAULT 'claude-opus-4-8',
    system_prompt       TEXT,
    max_tokens          INT         NOT NULL DEFAULT 4096,
    temperature         DOUBLE PRECISION NOT NULL DEFAULT 0.7,
    is_global_default   BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_provider_configs_org ON ai_provider_configs (organization_id);

-- One row with organization_id = NULL serves as the global admin default
-- Partial unique index ensures only one global default
CREATE UNIQUE INDEX idx_ai_provider_configs_global ON ai_provider_configs (is_global_default)
    WHERE is_global_default = TRUE;

ALTER TABLE ai_provider_configs ENABLE ROW LEVEL SECURITY;

-- Insert global default
INSERT INTO ai_provider_configs (is_global_default)
VALUES (TRUE);
