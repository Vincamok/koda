CREATE TABLE koda_instances (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name                    TEXT        NOT NULL UNIQUE,
    base_url                TEXT        NOT NULL,
    api_token_secret_ref_id UUID        REFERENCES secret_refs(id),
    region                  TEXT,
    status                  TEXT        NOT NULL DEFAULT 'unknown' CHECK (status IN ('healthy', 'degraded', 'unreachable', 'unknown')),
    last_seen_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_koda_instances_status ON koda_instances (status);

CREATE TABLE org_instance_affinities (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID        NOT NULL UNIQUE REFERENCES organizations(id),
    instance_id      UUID        NOT NULL REFERENCES koda_instances(id),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_instance_affinities_organization_id ON org_instance_affinities (organization_id);
CREATE INDEX idx_org_instance_affinities_instance_id     ON org_instance_affinities (instance_id);
