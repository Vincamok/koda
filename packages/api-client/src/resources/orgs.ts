import type { Organization, OrgMember } from '@koda/shared-types'
import type { KodaClient } from '../client'

export interface CreateOrgRequest {
  name: string
  slug: string
}

export interface InviteMemberRequest {
  email: string
  role: 'owner' | 'admin' | 'member'
}

export interface PatchMemberRequest {
  role: 'owner' | 'admin' | 'member'
}

export class OrgsResource {
  constructor(private readonly client: KodaClient) {}

  create(body: CreateOrgRequest): Promise<Organization> {
    return this.client.post('/api/v1/organizations', body)
  }

  get(orgId: string, signal?: AbortSignal): Promise<Organization> {
    return this.client.get(`/api/v1/organizations/${orgId}`, signal)
  }

  members(orgId: string, signal?: AbortSignal): Promise<OrgMember[]> {
    return this.client.get(`/api/v1/organizations/${orgId}/members`, signal)
  }

  inviteMember(orgId: string, body: InviteMemberRequest): Promise<OrgMember> {
    return this.client.post(`/api/v1/organizations/${orgId}/members`, body)
  }

  patchMember(orgId: string, userId: string, body: PatchMemberRequest): Promise<OrgMember> {
    return this.client.patch(`/api/v1/organizations/${orgId}/members/${userId}`, body)
  }

  removeMember(orgId: string, userId: string): Promise<void> {
    return this.client.delete(`/api/v1/organizations/${orgId}/members/${userId}`)
  }
}
