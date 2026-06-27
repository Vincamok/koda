import type { KodaClient } from '../client'

export interface UserSettings {
  user_id: string
  locale: string
  theme_id: string
  updated_at: string
}

export interface UpdateUserSettingsRequest {
  locale?: string
  theme_id?: string
}

export class UserSettingsResource {
  constructor(private readonly client: KodaClient) {}

  get(signal?: AbortSignal): Promise<UserSettings> {
    return this.client.get('/api/v1/user/settings', signal)
  }

  update(body: UpdateUserSettingsRequest): Promise<UserSettings> {
    return this.client.put('/api/v1/user/settings', body)
  }
}
