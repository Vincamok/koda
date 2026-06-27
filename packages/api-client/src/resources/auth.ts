import type { User } from '@koda/shared-types'
import type { KodaClient } from '../client'

export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
  display_name: string
}

export class AuthResource {
  constructor(private readonly client: KodaClient) {}

  me(signal?: AbortSignal): Promise<User> {
    return this.client.get('/api/v1/auth/me', signal)
  }

  login(body: LoginRequest): Promise<User> {
    return this.client.post('/api/v1/auth/login', body)
  }

  register(body: RegisterRequest): Promise<User> {
    return this.client.post('/api/v1/auth/register', body)
  }

  logout(): Promise<void> {
    return this.client.post('/api/v1/auth/logout', {})
  }

  oauthUrl(provider: 'google' | 'github' | 'authentik'): string {
    return `${(this.client as unknown as { baseUrl: string }).baseUrl}/api/v1/auth/oauth/${provider}`
  }
}
