export { KodaClient, KodaApiError } from './client'
export type { KodaClientOptions } from './client'

export { AuthResource } from './resources/auth'
export type { LoginRequest, RegisterRequest } from './resources/auth'

export { OrgsResource } from './resources/orgs'
export type { CreateOrgRequest, InviteMemberRequest, PatchMemberRequest } from './resources/orgs'

export { PersonalResource } from './resources/personal'
export type { PersonalSpace, Snippet, CreateSnippetRequest } from './resources/personal'

export { UserSettingsResource } from './resources/user-settings'
export type { UserSettings, UpdateUserSettingsRequest } from './resources/user-settings'

import { KodaClient } from './client'
import { AuthResource } from './resources/auth'
import { OrgsResource } from './resources/orgs'
import { PersonalResource } from './resources/personal'
import { UserSettingsResource } from './resources/user-settings'
import type { KodaClientOptions } from './client'

export interface KodaSDK {
  auth: AuthResource
  orgs: OrgsResource
  personal: PersonalResource
  userSettings: UserSettingsResource
}

export function createKodaClient(opts: KodaClientOptions): KodaSDK {
  const client = new KodaClient(opts)
  return {
    auth: new AuthResource(client),
    orgs: new OrgsResource(client),
    personal: new PersonalResource(client),
    userSettings: new UserSettingsResource(client),
  }
}
