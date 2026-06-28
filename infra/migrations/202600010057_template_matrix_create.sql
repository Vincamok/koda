-- Multi-runtime template matrix support
-- Each template can have multiple runtime variants (e.g. Node 18, Node 20, Python 3.11, Python 3.12)
CREATE TABLE template_runtime_variants (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id  UUID NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    runtime      TEXT NOT NULL,   -- e.g. 'node', 'python', 'go', 'rust', 'ruby', 'java'
    version      TEXT NOT NULL,   -- e.g. '20', '3.12', '1.21', '1.79'
    docker_image TEXT NOT NULL,   -- e.g. 'node:20-slim', 'python:3.12-slim'
    -- Override devcontainer config for this runtime variant
    devcontainer_override JSONB NOT NULL DEFAULT '{}',
    is_default   BOOL NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_template_variants_unique
    ON template_runtime_variants(template_id, runtime, version);

CREATE INDEX idx_template_variants_template ON template_runtime_variants(template_id);

-- Seed variants for common templates (multi-runtime Node + Python)
-- These are placeholders that get filled in when templates are created via the API
