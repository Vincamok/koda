CREATE TABLE themes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT,
    author      TEXT,
    preview_url TEXT,
    -- JSON design tokens (colors, fonts, spacing, etc.)
    tokens      JSONB NOT NULL DEFAULT '{}',
    is_builtin  BOOL NOT NULL DEFAULT false,
    source_url  TEXT,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Pre-populate the 4 built-in themes
INSERT INTO themes (slug, name, description, is_builtin, tokens) VALUES
    ('default',  'Default',  'The standard Koda theme',           true, '{"colorScheme":"dark","primaryColor":"#7C3AED"}'),
    ('minimal',  'Minimal',  'Clean and distraction-free',        true, '{"colorScheme":"dark","primaryColor":"#374151"}'),
    ('pro',      'Pro',      'High-contrast professional theme',  true, '{"colorScheme":"dark","primaryColor":"#1D4ED8"}'),
    ('light',    'Light',    'Light mode for bright environments',true, '{"colorScheme":"light","primaryColor":"#7C3AED"}');

CREATE INDEX idx_themes_slug ON themes(slug);
