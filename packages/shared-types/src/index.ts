// ── Core API response envelope ────────────────────────────────────────────────

export interface ApiResponse<T> {
  data: T
  meta?: Record<string, unknown>
}

export interface ApiError {
  error: {
    code: string
    message: string
    request_id: string
  }
}

export interface CursorPage<T> {
  data: T[]
  meta: {
    next_cursor: string | null
    has_more: boolean
  }
}

// ── User & Auth ───────────────────────────────────────────────────────────────

export interface User {
  id: string
  email: string
  display_name: string
  avatar_url: string | null
  email_verified: boolean
  is_super_admin: boolean
  created_at: string
}

export interface UserSettings {
  user_id: string
  locale: 'fr' | 'en' | 'es' | 'de'
  theme_id: string
  created_at: string
  updated_at: string
}

// ── Organization & Membership ─────────────────────────────────────────────────

export interface Organization {
  id: string
  name: string
  slug: string
  status: 'active' | 'suspended'
  created_at: string
}

export type OrgRole = 'owner' | 'admin' | 'member'

export interface Membership {
  user_id: string
  email: string
  display_name: string
  role: OrgRole
  joined_at: string
}

export interface OrganizationQuota {
  id: string
  organization_id: string
  max_workspaces: number
  max_cpu_cores: number
  max_ram_gb: number
  max_storage_gb: number
  max_members: number
}

// ── Teams ─────────────────────────────────────────────────────────────────────

export interface Team {
  id: string
  organization_id: string
  name: string
  description: string | null
  created_at: string
}

export type TeamRole = 'lead' | 'developer' | 'reviewer' | 'viewer'

export interface TeamMembership {
  team_id: string
  user_id: string
  email: string
  display_name: string
  role: TeamRole
}

export interface TeamQuota {
  team_id: string
  max_workspaces: number
  max_cpu_cores: number
  max_ram_gb: number
}

// ── Workspace ─────────────────────────────────────────────────────────────────

export type WorkspaceStatus =
  | 'created'
  | 'cloning'
  | 'ready'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'stopped'
  | 'reviewing'
  | 'closed'
  | 'failed'

export interface Workspace {
  id: string
  uid: string
  organization_id: string
  project_id: string | null
  template_id: string | null
  created_by: string
  name: string
  status: WorkspaceStatus
  cpu_limit: number
  ram_limit_mb: number
  created_at: string
  updated_at: string
}

export interface WorkspaceGitConfig {
  id: string
  workspace_id: string
  repo_url: string
  branch: string
  clone_status: 'pending' | 'cloning' | 'ready' | 'failed'
  clone_error: string | null
  last_cloned_at: string | null
}

// ── Plugins ───────────────────────────────────────────────────────────────────

export type PluginType = 'web' | 'tcp' | 'background'

export interface PluginDefinition {
  id: string
  slug: string
  name: string
  description: string | null
  version: string
  plugin_type: PluginType
  docker_image: string
  internal_port: number | null
  health_check_path: string | null
  is_builtin: boolean
}

export type BindingStatus = 'pending' | 'starting' | 'running' | 'unhealthy' | 'stopped' | 'failed'

export interface WorkspacePluginBinding {
  id: string
  uid: string
  workspace_id: string
  plugin_definition_id: string
  container_id: string | null
  status: BindingStatus
  config: Record<string, unknown>
}

// ── MCP ───────────────────────────────────────────────────────────────────────

export interface WorkspaceMCPBinding {
  id: string
  workspace_id: string
  connector_definition_id: string
  config: Record<string, unknown>
  secret_ref_ids: string[]
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface UserMCPBinding {
  id: string
  user_id: string
  connector_definition_id: string
  config: Record<string, unknown>
  secret_ref_ids: string[]
  enabled: boolean
  created_at: string
  updated_at: string
}

// ── Security ──────────────────────────────────────────────────────────────────

export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info'
export type ScanType = 'secret_scan' | 'sast' | 'dependency_scan' | 'image_scan'

export interface SecurityReport {
  id: string
  workspace_id: string
  scan_type: ScanType
  status: 'pending' | 'running' | 'completed' | 'failed'
  summary: string | null
  created_at: string
}

export interface VulnerabilityFinding {
  id: string
  security_report_id: string
  title: string
  description: string | null
  severity: Severity
  file_path: string | null
  line_number: number | null
  remediation: string | null
}

// ── Tickets ───────────────────────────────────────────────────────────────────

export interface TicketRecord {
  id: string
  workspace_id: string
  organization_id: string
  title: string
  description: string | null
  status: 'open' | 'in_progress' | 'closed'
  priority: 'critical' | 'high' | 'medium' | 'low'
  external_url: string | null
  external_system: string | null
  created_by: string
  created_at: string
}

// ── Audit ─────────────────────────────────────────────────────────────────────

export interface AuditEvent {
  id: string
  actor_id: string | null
  organization_id: string | null
  action: string
  resource_type: string | null
  resource_id: string | null
  metadata: Record<string, unknown>
  ip_address: string | null
  created_at: string
}

// ── Multi-instance ────────────────────────────────────────────────────────────

export interface KodaInstance {
  id: string
  name: string
  base_url: string
  region: string | null
  status: 'healthy' | 'degraded' | 'unreachable' | 'unknown'
  last_seen_at: string | null
  created_at: string
}
