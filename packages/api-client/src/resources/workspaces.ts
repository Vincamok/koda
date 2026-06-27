import type { Workspace, WorkspaceStatus } from '@koda/shared-types'
import type { KodaClient } from '../client'

export interface GitConfigRequest {
  repo_url: string
  branch?: string
  ssh_key_secret_ref_id?: string
}

export interface CreateWorkspaceRequest {
  name: string
  project_id?: string
  template_id?: string
  cpu_limit?: number
  ram_limit_mb?: number
  git?: GitConfigRequest
}

export interface ListWorkspacesQuery {
  project_id?: string
  status?: WorkspaceStatus
  limit?: number
  offset?: number
}

export class WorkspacesResource {
  constructor(private readonly client: KodaClient) {}

  list(orgId: string, query?: ListWorkspacesQuery, signal?: AbortSignal): Promise<Workspace[]> {
    const params = new URLSearchParams()
    if (query?.project_id) params.set('project_id', query.project_id)
    if (query?.status) params.set('status', query.status)
    if (query?.limit != null) params.set('limit', String(query.limit))
    if (query?.offset != null) params.set('offset', String(query.offset))
    const qs = params.toString() ? `?${params}` : ''
    return this.client.get(`/api/v1/organizations/${orgId}/workspaces${qs}`, signal)
  }

  create(orgId: string, body: CreateWorkspaceRequest): Promise<Workspace> {
    return this.client.post(`/api/v1/organizations/${orgId}/workspaces`, body)
  }

  get(orgId: string, workspaceId: string, signal?: AbortSignal): Promise<Workspace> {
    return this.client.get(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}`, signal)
  }

  start(orgId: string, workspaceId: string): Promise<{ status: string }> {
    return this.client.post(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}/start`, {})
  }

  stop(orgId: string, workspaceId: string): Promise<{ status: string }> {
    return this.client.post(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}/stop`, {})
  }

  delete(orgId: string, workspaceId: string): Promise<void> {
    return this.client.delete(`/api/v1/organizations/${orgId}/workspaces/${workspaceId}`)
  }
}
