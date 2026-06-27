import type { KodaClient } from '../client'

export interface PersonalSpace {
  id: string
  user_id: string
  volume_name: string
  created_at: string
}

export interface Snippet {
  id: string
  user_id: string
  language: string
  name: string
  content: string
  description: string | null
  created_at: string
}

export interface CreateSnippetRequest {
  language: string
  name: string
  content: string
  description?: string
}

export class PersonalResource {
  constructor(private readonly client: KodaClient) {}

  getSpace(signal?: AbortSignal): Promise<PersonalSpace> {
    return this.client.get('/api/v1/personal/space', signal)
  }

  snippets(signal?: AbortSignal): Promise<Snippet[]> {
    return this.client.get('/api/v1/personal/snippets', signal)
  }

  createSnippet(body: CreateSnippetRequest): Promise<Snippet> {
    return this.client.post('/api/v1/personal/snippets', body)
  }

  updateSnippet(snippetId: string, content: string): Promise<void> {
    return this.client.patch(`/api/v1/personal/snippets/${snippetId}`, { content })
  }

  deleteSnippet(snippetId: string): Promise<void> {
    return this.client.delete(`/api/v1/personal/snippets/${snippetId}`)
  }
}
