import { createKodaClient } from '@koda/api-client'

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'

export const api = createKodaClient({
  baseUrl: API_URL,
  onUnauthorized: () => {
    if (typeof window !== 'undefined') {
      window.location.href = '/login'
    }
  },
})
