CREATE TABLE users (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT        NOT NULL UNIQUE,
    email_verified  BOOL        NOT NULL DEFAULT false,
    password_hash   TEXT,
    display_name    TEXT        NOT NULL,
    avatar_url      TEXT,
    is_super_admin  BOOL        NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users (email);
