const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'

async function adminGet<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: 'include',
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!res.ok) throw new Error(`Admin API ${res.status}: ${path}`)
  const json = await res.json()
  return json.data ?? json
}

async function adminPost<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: body !== undefined ? JSON.stringify(body) : undefined,
    cache: 'no-store',
  })
  if (!res.ok) throw new Error(`Admin API ${res.status}: ${path}`)
  if (res.status === 204) return undefined as T
  const json = await res.json()
  return json.data ?? json
}

// ── Types ─────────────────────────────────────────────────────────────────────

export interface AdminStats {
  total_orgs: number
  active_orgs: number
  total_users: number
  total_workspaces: number
  running_workspaces: number
  total_pipelines: number
}

export interface AdminOrg {
  id: string
  name: string
  slug: string
  status: string
  created_at: string
  member_count: number
  workspace_count: number
}

export interface AdminUser {
  id: string
  email: string
  display_name: string
  is_super_admin: boolean
  email_verified: boolean
  created_at: string
  org_count: number
}

export interface AuditLogEntry {
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

export interface ImpersonateResponse {
  token: string
  expires_at: string
}

// ── API calls ─────────────────────────────────────────────────────────────────

export function getAdminStats(): Promise<AdminStats> {
  return adminGet<AdminStats>('/api/v1/admin/stats')
}

export function listAdminOrgs(page = 0): Promise<{ data: AdminOrg[]; total: number }> {
  return adminGet<{ data: AdminOrg[]; total: number }>(`/api/v1/admin/organizations?limit=20&offset=${page * 20}`)
}

export function toggleOrgStatus(orgId: string): Promise<AdminOrg> {
  return adminPost<AdminOrg>(`/api/v1/admin/organizations/${orgId}/toggle`)
}

export function listAdminUsers(page = 0): Promise<{ data: AdminUser[]; total: number }> {
  return adminGet<{ data: AdminUser[]; total: number }>(`/api/v1/admin/users?limit=20&offset=${page * 20}`)
}

export function impersonateUser(userId: string): Promise<ImpersonateResponse> {
  return adminPost<ImpersonateResponse>(`/api/v1/admin/users/${userId}/impersonate`)
}

export function listAuditLogs(page = 0): Promise<{ data: AuditLogEntry[]; total: number }> {
  return adminGet<{ data: AuditLogEntry[]; total: number }>(`/api/v1/admin/audit-logs?limit=50&offset=${page * 50}`)
}
