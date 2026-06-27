-- Enable Row Level Security on critical tables.
-- Real enforcement is handled at the application layer via organization_id filters.
-- These permissive policies allow all access for now and can be tightened later.

ALTER TABLE workspaces                ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects                  ENABLE ROW LEVEL SECURITY;
ALTER TABLE cicd_pipelines            ENABLE ROW LEVEL SECURITY;
ALTER TABLE security_reports          ENABLE ROW LEVEL SECURITY;
ALTER TABLE vulnerability_findings    ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_plugin_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE exposure_rules            ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_git_configs     ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_volumes         ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_shares          ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_mcp_bindings    ENABLE ROW LEVEL SECURITY;
ALTER TABLE ticket_records            ENABLE ROW LEVEL SECURITY;
ALTER TABLE secret_refs               ENABLE ROW LEVEL SECURITY;

CREATE POLICY "app_access" ON workspaces                USING (true);
CREATE POLICY "app_access" ON projects                  USING (true);
CREATE POLICY "app_access" ON cicd_pipelines            USING (true);
CREATE POLICY "app_access" ON security_reports          USING (true);
CREATE POLICY "app_access" ON vulnerability_findings    USING (true);
CREATE POLICY "app_access" ON workspace_plugin_bindings USING (true);
CREATE POLICY "app_access" ON exposure_rules            USING (true);
CREATE POLICY "app_access" ON workspace_git_configs     USING (true);
CREATE POLICY "app_access" ON workspace_volumes         USING (true);
CREATE POLICY "app_access" ON workspace_shares          USING (true);
CREATE POLICY "app_access" ON workspace_mcp_bindings    USING (true);
CREATE POLICY "app_access" ON ticket_records            USING (true);
CREATE POLICY "app_access" ON secret_refs               USING (true);
