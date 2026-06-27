import type { User } from '@koda/shared-types'
import { getMe, logout as apiLogout } from './api-client'

// ── Session ───────────────────────────────────────────────────────────────────

/**
 * Returns the current session user by calling the API.
 * Returns null if unauthenticated (catches 401 errors).
 */
export async function getSession(): Promise<User | null> {
  try {
    const user = await getMe()
    return user
  } catch {
    return null
  }
}

// ── Logout ────────────────────────────────────────────────────────────────────

/**
 * Signs the user out and redirects to the login page.
 */
export async function logout(locale: string = 'fr'): Promise<void> {
  try {
    await apiLogout()
  } catch {
    // Ignore errors — we still want to redirect
  }
  if (typeof window !== 'undefined') {
    window.location.href = `/${locale}/login`
  }
}
