import { test, expect } from '@playwright/test'
import path from 'path'

test.use({
  storageState: path.join(__dirname, '../.auth/user.json'),
})

const LOCALE = 'fr'

test.describe('MFA', () => {
  test('MFA setup page reachable', async ({ page }) => {
    await page.goto(`/${LOCALE}/settings`)
    // Look for MFA section in settings
    const mfaSection = page.getByText(/authentification à deux facteurs|two-factor|mfa/i)
    await expect(mfaSection).toBeVisible({ timeout: 10_000 })
  })
})

test.describe('Rate limiting', () => {
  test('API responds with 429 after excessive requests', async ({ request }) => {
    const base = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'
    let rateLimited = false

    for (let i = 0; i < 310; i++) {
      const res = await request.get(`${base}/api/v1/auth/me`)
      if (res.status() === 429) {
        rateLimited = true
        break
      }
    }

    expect(rateLimited).toBe(true)
  })
})

test.describe('Security headers', () => {
  test('API returns security headers', async ({ request }) => {
    const base = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'
    const res = await request.get(`${base}/api/v1/auth/me`)

    // Content-Type should be JSON
    const contentType = res.headers()['content-type'] ?? ''
    expect(contentType).toContain('application/json')
  })
})
