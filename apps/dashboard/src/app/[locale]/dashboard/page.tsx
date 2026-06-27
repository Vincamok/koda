import { getTranslations } from 'next-intl/server'
import Link from 'next/link'
import { Plus, Boxes, Activity } from 'lucide-react'
import { getSession } from '@/lib/auth'
import { listWorkspaces } from '@/lib/api-client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { WorkspaceCard } from '@/components/workspace/workspace-card'
import type { Workspace } from '@koda/shared-types'

interface DashboardPageProps {
  params: { locale: string }
}

export default async function DashboardPage({
  params: { locale },
}: DashboardPageProps) {
  const t = await getTranslations('dashboard')
  const wt = await getTranslations('workspace')
  const user = await getSession()

  let workspaces: Workspace[] = []
  let totalWorkspaces = 0
  let activeWorkspaces = 0

  if (user) {
    try {
      // Fetch workspaces for the first org available on user's account
      // In a real app this would use user's active org from session/cookie
      const page = await listWorkspaces('default')
      workspaces = page.data.slice(0, 6) // Show at most 6 recent
      totalWorkspaces = page.data.length
      activeWorkspaces = page.data.filter(
        (w) => w.status === 'running' || w.status === 'starting',
      ).length
    } catch {
      // API not available — show empty state
    }
  }

  return (
    <div className="space-y-6">
      {/* Welcome header */}
      <div className="flex flex-col gap-1">
        <h2 className="text-2xl font-bold text-koda-text">
          {t('welcome', { name: user?.display_name ?? '' })}
        </h2>
        <p className="text-sm text-koda-text-muted">{t('overview')}</p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-koda-text-muted">
              {t('active_workspaces')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <Activity className="h-5 w-5 text-green-400" />
              <span className="text-3xl font-bold text-koda-text">
                {activeWorkspaces}
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-koda-text-muted">
              {t('total_workspaces')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <Boxes className="h-5 w-5 text-koda-primary" />
              <span className="text-3xl font-bold text-koda-text">
                {totalWorkspaces}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Recent workspaces */}
      <div>
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-lg font-semibold text-koda-text">
            {t('recent_workspaces')}
          </h3>
          <Link href={`/${locale}/workspaces/new`}>
            <Button variant="primary" size="sm">
              <Plus className="h-4 w-4" />
              {wt('new')}
            </Button>
          </Link>
        </div>

        {workspaces.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-koda-border py-16 gap-4">
            <Boxes className="h-10 w-10 text-koda-text-muted" />
            <p className="text-sm text-koda-text-muted">{t('no_workspaces')}</p>
            <Link href={`/${locale}/workspaces/new`}>
              <Button variant="secondary" size="sm">
                <Plus className="h-4 w-4" />
                {t('create_first')}
              </Button>
            </Link>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
            {workspaces.map((ws) => (
              <WorkspaceCard key={ws.id} workspace={ws} locale={locale} />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
