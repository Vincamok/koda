-- Structured log entries for Loki shipping
CREATE TABLE log_entries (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ts         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level      TEXT NOT NULL DEFAULT 'info'
                   CHECK (level IN ('trace', 'debug', 'info', 'warn', 'error')),
    service    TEXT NOT NULL,
    message    TEXT NOT NULL,
    fields     JSONB,
    -- Set once the row has been shipped to Loki
    shipped_at TIMESTAMPTZ
);

CREATE INDEX idx_log_entries_unshipped ON log_entries(ts) WHERE shipped_at IS NULL;
CREATE INDEX idx_log_entries_service   ON log_entries(service, ts DESC);

-- Auto-purge entries older than 30 days (Loki keeps them long-term)
CREATE OR REPLACE FUNCTION purge_old_log_entries() RETURNS void LANGUAGE sql AS $$
    DELETE FROM log_entries WHERE ts < NOW() - INTERVAL '30 days' AND shipped_at IS NOT NULL;
$$;
