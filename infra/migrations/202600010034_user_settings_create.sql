CREATE TABLE user_settings (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL UNIQUE REFERENCES users(id),
    locale      TEXT        NOT NULL DEFAULT 'fr' CHECK (locale IN ('fr', 'en', 'es', 'de')),
    theme_id    TEXT        NOT NULL DEFAULT 'default',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_settings_user_id ON user_settings (user_id);
