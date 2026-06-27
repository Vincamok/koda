import Link from 'next/link'
import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'
import { listAdminOrgs } from '@/lib/admin-api'
import type { AdminOrg } from '@/lib/admin-api'

interface Props {
  params: { locale: string }
  searchParams: { page?: string }
}

function StatusBadge({ status }: { status: string }) {
  return (
    <span
      className={`inline-block rounded px-2 py-0.5 text-xs font-semibold uppercase ${
        status === 'active'
          ? 'bg-emerald-900/40 text-emerald-300'
          : 'bg-red-900/40 text-red-300'
      }`}
    >
      {status}
    </span>
  )
}

export default async function OrgsPage({ params: { locale }, searchParams }: Props) {
  const user = await getAdminSession()
  if (!user) redirect(`/${locale}/login`)

  const page = parseInt(searchParams.page ?? '0', 10)
  let orgs: AdminOrg[] = []
  let total = 0

  try {
    const result = await listAdminOrgs(page)
    orgs = result.data
    total = result.total
  } catch {
    // API unavailable
  }

  const totalPages = Math.ceil(total / 20)

  return (
    <div className="min-h-screen bg-zinc-950 p-8">
      <div className="mb-6">
        <Link href={`/${locale}/dashboard`} className="text-sm text-zinc-400 hover:text-zinc-100">
          ← Dashboard
        </Link>
        <div className="mt-2 flex items-center justify-between">
          <h1 className="text-2xl font-bold text-zinc-100">Organisations</h1>
          <span className="text-sm text-zinc-500">{total} au total</span>
        </div>
      </div>

      <div className="rounded-xl border border-zinc-800 overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-zinc-900 text-left">
            <tr>
              <th className="px-4 py-3 font-medium text-zinc-400">Nom</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Slug</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Statut</th>
              <th className="px-4 py-3 font-medium text-zinc-400 text-right">Membres</th>
              <th className="px-4 py-3 font-medium text-zinc-400 text-right">Workspaces</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Créée le</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-zinc-800">
            {orgs.length === 0 ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-zinc-500">
                  Aucune organisation
                </td>
              </tr>
            ) : (
              orgs.map((org) => (
                <tr key={org.id} className="bg-zinc-950 hover:bg-zinc-900/50 transition-colors">
                  <td className="px-4 py-3 font-medium text-zinc-200">{org.name}</td>
                  <td className="px-4 py-3 text-zinc-400 font-mono text-xs">{org.slug}</td>
                  <td className="px-4 py-3">
                    <StatusBadge status={org.status} />
                  </td>
                  <td className="px-4 py-3 text-right text-zinc-400">{org.member_count}</td>
                  <td className="px-4 py-3 text-right text-zinc-400">{org.workspace_count}</td>
                  <td className="px-4 py-3 text-zinc-400 text-xs">
                    {new Date(org.created_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <form action={`/api/admin/orgs/${org.id}/toggle`} method="POST">
                      <button
                        type="submit"
                        className={`text-xs rounded px-2.5 py-1 font-medium transition-colors ${
                          org.status === 'active'
                            ? 'bg-red-900/30 text-red-300 hover:bg-red-900/50'
                            : 'bg-emerald-900/30 text-emerald-300 hover:bg-emerald-900/50'
                        }`}
                      >
                        {org.status === 'active' ? 'Suspendre' : 'Réactiver'}
                      </button>
                    </form>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-between text-sm text-zinc-400">
          <span>Page {page + 1} / {totalPages}</span>
          <div className="flex gap-2">
            {page > 0 && (
              <Link
                href={`/${locale}/orgs?page=${page - 1}`}
                className="rounded border border-zinc-700 px-3 py-1 hover:border-zinc-500 transition-colors"
              >
                Précédent
              </Link>
            )}
            {page < totalPages - 1 && (
              <Link
                href={`/${locale}/orgs?page=${page + 1}`}
                className="rounded border border-zinc-700 px-3 py-1 hover:border-zinc-500 transition-colors"
              >
                Suivant
              </Link>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
