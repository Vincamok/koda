import Link from 'next/link'
import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'
import { listAdminUsers } from '@/lib/admin-api'
import type { AdminUser } from '@/lib/admin-api'

interface Props {
  params: { locale: string }
  searchParams: { page?: string }
}

export default async function UsersPage({ params: { locale }, searchParams }: Props) {
  const user = await getAdminSession()
  if (!user) redirect(`/${locale}/login`)

  const page = parseInt(searchParams.page ?? '0', 10)
  let users: AdminUser[] = []
  let total = 0

  try {
    const result = await listAdminUsers(page)
    users = result.data
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
          <h1 className="text-2xl font-bold text-zinc-100">Utilisateurs</h1>
          <span className="text-sm text-zinc-500">{total} au total</span>
        </div>
      </div>

      <div className="rounded-xl border border-zinc-800 overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-zinc-900 text-left">
            <tr>
              <th className="px-4 py-3 font-medium text-zinc-400">Email</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Nom</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Rôle</th>
              <th className="px-4 py-3 font-medium text-zinc-400 text-right">Orgs</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Vérifié</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Créé le</th>
              <th className="px-4 py-3 font-medium text-zinc-400">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-zinc-800">
            {users.length === 0 ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-zinc-500">
                  Aucun utilisateur
                </td>
              </tr>
            ) : (
              users.map((u) => (
                <tr key={u.id} className="bg-zinc-950 hover:bg-zinc-900/50 transition-colors">
                  <td className="px-4 py-3 text-zinc-200">{u.email}</td>
                  <td className="px-4 py-3 text-zinc-400">{u.display_name}</td>
                  <td className="px-4 py-3">
                    {u.is_super_admin ? (
                      <span className="inline-block rounded bg-indigo-900/40 px-2 py-0.5 text-xs font-semibold uppercase text-indigo-300">
                        super admin
                      </span>
                    ) : (
                      <span className="text-xs text-zinc-500">user</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-right text-zinc-400">{u.org_count}</td>
                  <td className="px-4 py-3">
                    {u.email_verified ? (
                      <span className="text-xs text-emerald-400">✓</span>
                    ) : (
                      <span className="text-xs text-yellow-500">—</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-zinc-400 text-xs">
                    {new Date(u.created_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <form action={`/api/admin/users/${u.id}/impersonate`} method="POST">
                      <button
                        type="submit"
                        className="text-xs rounded bg-amber-900/30 px-2.5 py-1 font-medium text-amber-300 hover:bg-amber-900/50 transition-colors"
                        title="Impersonation — tous les logs sont tracés"
                      >
                        Impersonner
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
              <Link href={`/${locale}/users?page=${page - 1}`} className="rounded border border-zinc-700 px-3 py-1 hover:border-zinc-500 transition-colors">
                Précédent
              </Link>
            )}
            {page < totalPages - 1 && (
              <Link href={`/${locale}/users?page=${page + 1}`} className="rounded border border-zinc-700 px-3 py-1 hover:border-zinc-500 transition-colors">
                Suivant
              </Link>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
