CREATE TABLE personal_snippets (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        NOT NULL REFERENCES users(id),
    language     TEXT        NOT NULL,
    name         TEXT        NOT NULL,
    content      TEXT        NOT NULL,
    description  TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_personal_snippets_user_id  ON personal_snippets (user_id);
CREATE INDEX idx_personal_snippets_language ON personal_snippets (language);
