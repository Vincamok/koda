import type {
  User,
  UserSettings,
  Workspace,
  CursorPage,
  ApiResponse,
  ApiError,
  Pipeline,
  PipelineRun,
  AutomationTrigger,
  IncomingWebhookEvent,
  SecurityReport,
  VulnerabilityFinding,
  AuditEvent,
} from '@koda/shared-types'

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'

// ── Error class ───────────────────────────────────────────────────────────────

export class ApiRequestError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    public readonly requestId: string,
    message: string,
  ) {
    super(message)
    this.name = 'ApiRequestError'
  }
}

// ── Locale helper (reads from cookie set by middleware) ───────────────────────

function getCurrentLocale(): string {
  if (typeof document === 'undefined') return 'fr'
  const match = document.cookie.match(/(?:^|;\s*)NEXT_LOCALE=([^;]+)/)
  return match?.[1] ?? 'fr'
}

// ── Core fetch wrapper ────────────────────────────────────────────────────────

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const url = `${API_BASE_URL}${path}`
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    Accept: 'application/json',
  }

  const res = await fetch(url, {
    method,
    headers,
    credentials: 'include',
    body: body !== undefined ? JSON.stringify(body) : undefined,
  })

  if (res.status === 401) {
    // Redirect to login, preserving locale
    if (typeof window !== 'undefined') {
      const locale = getCurrentLocale()
      window.location.href = `/${locale}/login`
    }
    throw new ApiRequestError(401, 'UNAUTHORIZED', '', 'Unauthorized')
  }

  if (!res.ok) {
    let errorPayload: ApiError | null = null
    try {
      errorPayload = await res.json()
    } catch {
      // ignore parse errors
    }
    const code = errorPayload?.error?.code ?? 'UNKNOWN'
    const message = errorPayload?.error?.message ?? res.statusText
    const requestId = errorPayload?.error?.request_id ?? ''
    throw new ApiRequestError(res.status, code, requestId, message)
  }

  // 204 No Content
  if (res.status === 204) {
    return undefined as T
  }

  const json: ApiResponse<T> = await res.json()
  return json.data
}

// ── HTTP method helpers ───────────────────────────────────────────────────────

export function get<T>(path: string): Promise<T> {
  return request<T>('GET', path)
}

export function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>('POST', path, body)
}

export function put<T>(path: string, body?: unknown): Promise<T> {
  return request<T>('PUT', path, body)
}

export function patch<T>(path: string, body?: unknown): Promise<T> {
  return request<T>('PATCH', path, body)
}

export function del<T>(path: string): Promise<T> {
  return request<T>('DELETE', path)
}

// ── Auth ──────────────────────────────────────────────────────────────────────

export function getMe(): Promise<User> {
  return get<User>('/api/v1/me')
}

export function login(email: string, password: string): Promise<User> {
  return post<User>('/api/v1/auth/login', { email, password })
}

export function register(
  email: string,
  password: string,
  displayName: string,
): Promise<User> {
  return post<User>('/api/v1/auth/register', {
    email,
    password,
    display_name: displayName,
  })
}

export async function logout(): Promise<void> {
  await post<void>('/api/v1/auth/logout')
}

// ── Workspaces ────────────────────────────────────────────────────────────────

export function listWorkspaces(
  orgId: string,
  cursor?: string,
): Promise<CursorPage<Workspace>> {
  const params = new URLSearchParams()
  if (cursor) params.set('cursor', cursor)
  const query = params.toString()
  const path = `/api/v1/orgs/${orgId}/workspaces${query ? `?${query}` : ''}`
  return get<CursorPage<Workspace>>(path)
}

export interface CreateWorkspaceData {
  name: string
  git_url?: string
  branch?: string
  template_id?: string
}

export function createWorkspace(
  orgId: string,
  data: CreateWorkspaceData,
): Promise<Workspace> {
  return post<Workspace>(`/api/v1/orgs/${orgId}/workspaces`, data)
}

// ── Pipelines ─────────────────────────────────────────────────────────────────

export function listPipelines(orgId: string, workspaceId: string): Promise<Pipeline[]> {
  return get<Pipeline[]>(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}/pipelines`)
}

export function createPipeline(
  orgId: string,
  workspaceId: string,
  data: { name: string; pipeline_type: string; config?: Record<string, unknown> },
): Promise<Pipeline> {
  return post<Pipeline>(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}/pipelines`, data)
}

export function runPipeline(orgId: string, workspaceId: string, pipelineId: string): Promise<PipelineRun> {
  return post<PipelineRun>(
    `/api/v1/organizations/${orgId}/workspaces/${workspaceId}/pipelines/${pipelineId}/run`,
  )
}

export function deletePipeline(orgId: string, workspaceId: string, pipelineId: string): Promise<void> {
  return del<void>(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}/pipelines/${pipelineId}`)
}

export function listTriggers(orgId: string, workspaceId: string): Promise<AutomationTrigger[]> {
  return get<AutomationTrigger[]>(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}/triggers`)
}

export function createTrigger(
  orgId: string,
  workspaceId: string,
  data: { pipeline_id: string; trigger_type: string; schedule_cron?: string },
): Promise<AutomationTrigger> {
  return post<AutomationTrigger>(
    `/api/v1/organizations/${orgId}/workspaces/${workspaceId}/triggers`,
    data,
  )
}

export function listWebhookEvents(orgId: string, workspaceId: string): Promise<IncomingWebhookEvent[]> {
  return get<IncomingWebhookEvent[]>(
    `/api/v1/organizations/${orgId}/workspaces/${workspaceId}/webhook-events`,
  )
}

export function listSecurityReports(orgId: string, workspaceId: string): Promise<SecurityReport[]> {
  return get<SecurityReport[]>(
    `/api/v1/organizations/${orgId}/workspaces/${workspaceId}/security-reports`,
  )
}

export function getAuditLogs(orgId: string, cursor?: string): Promise<CursorPage<AuditEvent>> {
  const params = cursor ? `?cursor=${encodeURIComponent(cursor)}` : ''
  return get<CursorPage<AuditEvent>>(`/api/v1/organizations/${orgId}/audit-logs${params}`)
}

// ── User settings ─────────────────────────────────────────────────────────────

export function getUserSettings(): Promise<UserSettings> {
  return get<UserSettings>('/api/v1/me/settings')
}

export function updateUserSettings(
  data: Partial<Pick<UserSettings, 'locale' | 'theme_id'>>,
): Promise<UserSettings> {
  return patch<UserSettings>('/api/v1/me/settings', data)
}
