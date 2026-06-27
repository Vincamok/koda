CREATE TABLE security_policies (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id       UUID        NOT NULL UNIQUE REFERENCES organizations(id),
    min_severity_to_block TEXT        NOT NULL DEFAULT 'critical' CHECK (min_severity_to_block IN ('critical', 'high', 'medium', 'low', 'none')),
    image_scan_trigger    TEXT        NOT NULL DEFAULT 'OnBuild' CHECK (image_scan_trigger IN ('OnBuild', 'OnLaunch', 'Both', 'Disabled')),
    security_ai_config    JSONB       NOT NULL DEFAULT '{}',
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_policies_organization_id ON security_policies (organization_id);
