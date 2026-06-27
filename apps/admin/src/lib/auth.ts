import { cookies } from 'next/headers'
import type { User } from '@koda/shared-types'

const API_URL = process.env.API_URL ?? 'http://localhost:8080'

export async function getAdminSession(): Promise<User | null> {
  const cookieStore = cookies()
  const sessionCookie = cookieStore.get('koda_session')
  if (!sessionCookie) return null

  try {
    const res = await fetch(`${API_URL}/api/v1/auth/me`, {
      headers: { Cookie: `koda_session=${sessionCookie.value}` },
      cache: 'no-store',
    })
    if (!res.ok) return null
    const json = await res.json()
    const user: User = json?.data ?? json
    if (!user.is_super_admin) return null
    return user
  } catch {
    return null
  }
}
