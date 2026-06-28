-- Extend plugin_definitions with marketplace fields
ALTER TABLE plugin_definitions
    ADD COLUMN IF NOT EXISTS author           TEXT,
    ADD COLUMN IF NOT EXISTS version          TEXT,
    ADD COLUMN IF NOT EXISTS category         TEXT,
    ADD COLUMN IF NOT EXISTS icon_url         TEXT,
    ADD COLUMN IF NOT EXISTS repo_url         TEXT,
    ADD COLUMN IF NOT EXISTS config_schema    JSONB NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS approved         BOOL NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS submitted_by_org  UUID REFERENCES organizations(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS submitted_by_user UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_plugin_definitions_approved ON plugin_definitions(approved) WHERE approved = true;
