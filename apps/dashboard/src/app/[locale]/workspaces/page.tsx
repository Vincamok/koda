import { getTranslations } from 'next-intl/server'
import Link from 'next/link'
import { redirect } from 'next/navigation'
import { Plus, Boxes } from 'lucide-react'
import { getSession } from '@/lib/auth'
import { listWorkspaces } from '@/lib/api-client'
import { AppShell } from '@/components/layout/app-shell'
import { Button } from '@/components/ui/button'
import { WorkspaceCard } from '@/components/workspace/workspace-card'
import type { Workspace } from '@koda/shared-types'

interface WorkspacesPageProps {
  params: { locale: string }
  searchParams: { cursor?: string }
}

export default async function WorkspacesPage({
  params: { locale },
  searchParams,
}: WorkspacesPageProps) {
  const t = await getTranslations('workspace')
  const user = await getSession()

  if (!user) {
    redirect(`/${locale}/login`)
  }

  const orgId = 'default' // TODO: persist org_id in session

  let workspaces: Workspace[] = []
  let nextCursor: string | null = null
  let hasMore = false

  try {
    const page = await listWorkspaces(orgId, searchParams.cursor)
    workspaces = page.data
    nextCursor = page.meta.next_cursor
    hasMore = page.meta.has_more
  } catch {
    // API not available — show empty state
  }

  return (
    <AppShell user={user!} locale={locale} title={t('title')}>
      <div className="space-y-6">
        {/* Page header */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-koda-text">{t('title')}</h2>
            <p className="mt-1 text-sm text-koda-text-muted">
              {workspaces.length} workspace{workspaces.length !== 1 ? 's' : ''}
            </p>
          </div>
          <Link href={`/${locale}/workspaces/new`}>
            <Button variant="primary" size="md">
              <Plus className="h-4 w-4" />
              {t('new')}
            </Button>
          </Link>
        </div>

        {/* Workspace grid */}
        {workspaces.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-koda-border py-20 gap-4">
            <Boxes className="h-12 w-12 text-koda-text-muted" />
            <div className="text-center">
              <p className="text-base font-medium text-koda-text">{t('empty')}</p>
              <p className="mt-1 text-sm text-koda-text-muted">
                {t('empty_description')}
              </p>
            </div>
            <Link href={`/${locale}/workspaces/new`}>
              <Button variant="primary" size="md">
                <Plus className="h-4 w-4" />
                {t('new')}
              </Button>
            </Link>
          </div>
        ) : (
          <>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
              {workspaces.map((ws) => (
                <WorkspaceCard key={ws.id} workspace={ws} locale={locale} orgId={orgId} />
              ))}
            </div>

            {/* Pagination */}
            {hasMore && nextCursor && (
              <div className="flex justify-center pt-4">
                <Link
                  href={`/${locale}/workspaces?cursor=${encodeURIComponent(nextCursor)}`}
                >
                  <Button variant="secondary" size="md">
                    Charger plus
                  </Button>
                </Link>
              </div>
            )}
          </>
        )}
      </div>
    </AppShell>
  )
}
