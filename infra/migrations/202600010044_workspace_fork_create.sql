ALTER TABLE workspaces ADD COLUMN IF NOT EXISTS forked_from UUID REFERENCES workspaces(id);
