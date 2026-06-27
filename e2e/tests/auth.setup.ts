import { test as setup, expect } from '@playwright/test'
import fs from 'fs'
import path from 'path'

const AUTH_FILE = path.join(__dirname, '../.auth/user.json')

const TEST_EMAIL = process.env.TEST_USER_EMAIL ?? 'test@koda.dev'
const TEST_PASSWORD = process.env.TEST_USER_PASSWORD ?? 'TestPassword123!'

setup('authenticate', async ({ page }) => {
  await page.goto('/fr/login')

  await page.getByLabel(/email/i).fill(TEST_EMAIL)
  await page.getByLabel(/mot de passe|password/i).fill(TEST_PASSWORD)
  await page.getByRole('button', { name: /connexion|sign in|login/i }).click()

  // Wait for redirect to dashboard or workspaces
  await expect(page).toHaveURL(/\/(dashboard|workspaces)/, { timeout: 10_000 })

  fs.mkdirSync(path.dirname(AUTH_FILE), { recursive: true })
  await page.context().storageState({ path: AUTH_FILE })
})
