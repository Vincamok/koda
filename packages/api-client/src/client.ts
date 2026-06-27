export interface KodaClientOptions {
  baseUrl: string
  onUnauthorized?: () => void
}

export class KodaApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly status: number,
    public readonly requestId?: string,
  ) {
    super(message)
    this.name = 'KodaApiError'
  }
}

export class KodaClient {
  private readonly baseUrl: string
  private readonly onUnauthorized?: () => void

  constructor(opts: KodaClientOptions) {
    this.baseUrl = opts.baseUrl.replace(/\/$/, '')
    this.onUnauthorized = opts.onUnauthorized
  }

  async request<T>(
    method: string,
    path: string,
    body?: unknown,
    signal?: AbortSignal,
  ): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method,
      credentials: 'include',
      headers: body ? { 'Content-Type': 'application/json' } : {},
      body: body ? JSON.stringify(body) : undefined,
      signal,
    })

    if (res.status === 401) {
      this.onUnauthorized?.()
      throw new KodaApiError('UNAUTHORIZED', 'Unauthorized', 401)
    }

    const json = await res.json().catch(() => null)

    if (!res.ok) {
      const err = json?.error
      throw new KodaApiError(
        err?.code ?? 'UNKNOWN',
        err?.message ?? res.statusText,
        res.status,
        err?.request_id,
      )
    }

    return (json?.data ?? json) as T
  }

  get<T>(path: string, signal?: AbortSignal) {
    return this.request<T>('GET', path, undefined, signal)
  }

  post<T>(path: string, body: unknown) {
    return this.request<T>('POST', path, body)
  }

  put<T>(path: string, body: unknown) {
    return this.request<T>('PUT', path, body)
  }

  patch<T>(path: string, body: unknown) {
    return this.request<T>('PATCH', path, body)
  }

  delete<T>(path: string) {
    return this.request<T>('DELETE', path)
  }
}
