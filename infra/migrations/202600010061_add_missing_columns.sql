-- Add metadata column to workspaces (JSONB for devcontainer config and other metadata)
ALTER TABLE workspaces ADD COLUMN IF NOT EXISTS metadata JSONB;

-- Add encryption_key_id to secret_refs (references encryption key used)
ALTER TABLE secret_refs ADD COLUMN IF NOT EXISTS encryption_key_id UUID;

-- Add status to vulnerability_findings
ALTER TABLE vulnerability_findings ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'open';

-- Add health_status to workspace_plugin_bindings
ALTER TABLE workspace_plugin_bindings ADD COLUMN IF NOT EXISTS health_status TEXT NOT NULL DEFAULT 'unknown';

-- Add last_probed_at to workspace_plugin_bindings
ALTER TABLE workspace_plugin_bindings ADD COLUMN IF NOT EXISTS last_probed_at TIMESTAMPTZ;
