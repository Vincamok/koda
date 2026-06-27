import { test, expect } from '@playwright/test'
import path from 'path'

test.use({
  storageState: path.join(__dirname, '../.auth/user.json'),
})

const LOCALE = 'fr'

test.describe('Workspace lifecycle', () => {
  test.describe.configure({ mode: 'serial' })
  let workspaceId: string

  test('create workspace', async ({ page }) => {
    await page.goto(`/${LOCALE}/workspaces/new`)

    // Fill the creation form
    await page.getByLabel(/^nom$|workspace name/i).fill('E2E Test Workspace')

    const submitBtn = page.getByRole('button', { name: /créer|create/i })
    await submitBtn.click()

    // Should redirect to workspace list or detail
    await expect(page).toHaveURL(/\/workspaces/, { timeout: 15_000 })

    // Capture workspace id from URL if redirected to detail page
    const url = page.url()
    const match = url.match(/\/workspaces\/([0-9a-f-]{36})/)
    if (match) workspaceId = match[1]
  })

  test('workspace appears in list', async ({ page }) => {
    await page.goto(`/${LOCALE}/workspaces`)

    await expect(page.getByText('E2E Test Workspace')).toBeVisible({ timeout: 10_000 })
  })

  test('view workspace pipelines tab', async ({ page }) => {
    if (!workspaceId) test.skip()

    await page.goto(`/${LOCALE}/workspaces/${workspaceId}?tab=pipelines`)

    await expect(page.getByText('Pipelines CI/CD')).toBeVisible()
    await expect(page.getByText('Nouveau pipeline')).toBeVisible()
  })

  test('view workspace webhooks tab', async ({ page }) => {
    if (!workspaceId) test.skip()

    await page.goto(`/${LOCALE}/workspaces/${workspaceId}?tab=webhooks`)

    await expect(page.getByText('Événements Webhook')).toBeVisible()
  })

  test('view workspace security tab', async ({ page }) => {
    if (!workspaceId) test.skip()

    await page.goto(`/${LOCALE}/workspaces/${workspaceId}?tab=security`)

    await expect(page.getByText('Rapports de sécurité')).toBeVisible()
  })
})

test.describe('Diff review', () => {
  test('workspace IDE loads for running workspace', async ({ page }) => {
    // Navigate to workspace list to find a running workspace
    await page.goto(`/${LOCALE}/workspaces`)

    const runningCard = page.locator('[data-status="running"]').first()
    const hasRunning = await runningCard.count()

    if (!hasRunning) {
      test.skip()
    }

    await runningCard.click()
    await expect(page).toHaveURL(/\/workspaces\/[0-9a-f-]{36}/, { timeout: 10_000 })
  })
})

test.describe('Settings', () => {
  test('user settings page loads', async ({ page }) => {
    await page.goto(`/${LOCALE}/settings`)
    await expect(page.locator('h1, h2').first()).toBeVisible()
  })
})
