import Link from 'next/link'
import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'
import { listAuditLogs } from '@/lib/admin-api'
import type { AuditLogEntry } from '@/lib/admin-api'

interface Props {
  params: { locale: string }
  searchParams: { page?: string }
}

const ACTION_COLORS: Record<string, string> = {
  'user.impersonate': 'text-amber-400',
  'mfa.enabled': 'text-emerald-400',
  'mfa.disabled': 'text-red-400',
  'workspace.start': 'text-blue-400',
  'workspace.stop': 'text-yellow-400',
  'workspace.delete': 'text-red-400',
  'org.created': 'text-emerald-400',
  'org.suspended': 'text-red-400',
}

export default async function AuditLogsPage({ params: { locale }, searchParams }: Props) {
  const user = await getAdminSession()
  if (!user) redirect(`/${locale}/login`)

  const page = parseInt(searchParams.page ?? '0', 10)
  let entries: AuditLogEntry[] = []
  let total = 0

  try {
    const result = await listAuditLogs(page)
    entries = result.data
    total = result.total
  } catch {
    // API unavailable
  }

  const totalPages = Math.ceil(total / 50)

  return (
    <div className="min-h-screen bg-zinc-950 p-8">
      <div className="mb-6">
        <Link href={`/${locale}/dashboard`} className="text-sm text-zinc-400 hover:text-zinc-100">
          ← Dashboard
        </Link>
        <div className="mt-2 flex items-center justify-between">
          <h1 className="text-2xl font-bold text-zinc-100">Logs d&apos;audit</h1>
          <span className="text-sm text-zinc-500">{total} entrées</span>
        </div>
      </div>

      <div className="space-y-1">
        {entries.length === 0 ? (
          <div className="rounded-xl border border-zinc-800 bg-zinc-900 px-4 py-8 text-center text-sm text-zinc-500">
            Aucune entrée d&apos;audit
          </div>
        ) : (
          entries.map((entry) => (
            <div
              key={entry.id}
              className="flex items-start gap-4 rounded-lg border border-zinc-800 bg-zinc-900 px-4 py-3"
            >
              <div className="w-44 shrink-0">
                <p className="text-xs text-zinc-500">
                  {new Date(entry.created_at).toLocaleString()}
                </p>
                {entry.ip_address && (
                  <p className="text-xs text-zinc-600 font-mono">{entry.ip_address}</p>
                )}
              </div>
              <div className="flex-1 min-w-0">
                <span
                  className={`font-mono text-sm font-medium ${ACTION_COLORS[entry.action] ?? 'text-zinc-300'}`}
                >
                  {entry.action}
                </span>
                {entry.resource_type && (
                  <span className="ml-2 text-xs text-zinc-500">
                    {entry.resource_type}
                    {entry.resource_id ? `:${entry.resource_id.slice(0, 8)}` : ''}
                  </span>
                )}
                {entry.actor_id && (
                  <p className="mt-0.5 text-xs text-zinc-600 font-mono">
                    actor: {entry.actor_id.slice(0, 8)}…
                  </p>
                )}
                {Object.keys(entry.metadata).length > 0 && (
                  <pre className="mt-1 text-[10px] text-zinc-600 overflow-hidden text-ellipsis whitespace-nowrap max-w-lg">
                    {JSON.stringify(entry.metadata)}
                  </pre>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-between text-sm text-zinc-400">
          <span>Page {page + 1} / {totalPages}</span>
          <div className="flex gap-2">
            {page > 0 && (
              <Link href={`/${locale}/audit-logs?page=${page - 1}`} className="rounded border border-zinc-700 px-3 py-1 hover:border-zinc-500 transition-colors">
                Précédent
              </Link>
            )}
            {page < totalPages - 1 && (
              <Link href={`/${locale}/audit-logs?page=${page + 1}`} className="rounded border border-zinc-700 px-3 py-1 hover:border-zinc-500 transition-colors">
                Suivant
              </Link>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
