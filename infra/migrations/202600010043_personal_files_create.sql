CREATE TABLE personal_files (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    path        TEXT        NOT NULL,
    content     TEXT        NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, path)
);

CREATE INDEX idx_personal_files_user_id ON personal_files (user_id);

ALTER TABLE personal_files ENABLE ROW LEVEL SECURITY;
