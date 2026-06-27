import Link from 'next/link'
import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'
import { getAdminStats } from '@/lib/admin-api'
import type { AdminStats } from '@/lib/admin-api'

interface Props {
  params: { locale: string }
}

function StatCard({ label, value, sub }: { label: string; value: number | string; sub?: string }) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
      <p className="text-xs font-medium uppercase tracking-wide text-zinc-500">{label}</p>
      <p className="mt-1.5 text-3xl font-bold text-zinc-100">{value}</p>
      {sub && <p className="mt-0.5 text-xs text-zinc-500">{sub}</p>}
    </div>
  )
}

function NavCard({ title, description, href }: { title: string; description: string; href: string }) {
  return (
    <Link
      href={href}
      className="group block rounded-xl border border-zinc-800 bg-zinc-900 p-6 transition-colors hover:border-indigo-700 hover:bg-zinc-800/60"
    >
      <h2 className="font-semibold text-zinc-100 group-hover:text-indigo-300 transition-colors">
        {title}
      </h2>
      <p className="mt-1 text-sm text-zinc-400">{description}</p>
    </Link>
  )
}

export default async function AdminDashboard({ params: { locale } }: Props) {
  const user = await getAdminSession()
  if (!user) redirect(`/${locale}/login`)

  let stats: AdminStats | null = null
  try {
    stats = await getAdminStats()
  } catch {
    // API unavailable
  }

  return (
    <div className="min-h-screen bg-zinc-950 p-8">
      <header className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-zinc-100">Koda Admin</h1>
          <p className="mt-0.5 text-sm text-zinc-400">Connecté en tant que {user.email}</p>
        </div>
        <form action={`/api/logout`} method="POST">
          <button
            type="submit"
            className="rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:border-zinc-500 hover:text-zinc-100 transition-colors"
          >
            Déconnexion
          </button>
        </form>
      </header>

      {/* Stats */}
      {stats && (
        <div className="mb-8 grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
          <StatCard label="Organisations" value={stats.total_orgs} sub={`${stats.active_orgs} actives`} />
          <StatCard label="Utilisateurs" value={stats.total_users} />
          <StatCard label="Workspaces" value={stats.total_workspaces} sub={`${stats.running_workspaces} en cours`} />
          <StatCard label="Pipelines" value={stats.total_pipelines} />
        </div>
      )}

      {/* Navigation */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <NavCard
          title="Utilisateurs"
          description="Gérer les utilisateurs, super-admins, impersonation"
          href={`/${locale}/users`}
        />
        <NavCard
          title="Organisations"
          description="Voir et gérer toutes les organisations"
          href={`/${locale}/orgs`}
        />
        <NavCard
          title="Logs d'audit"
          description="Historique complet des actions critiques"
          href={`/${locale}/audit-logs`}
        />
        <NavCard
          title="Infra"
          description="Workspaces actifs, conteneurs Docker, volumes"
          href={`/${locale}/infra`}
        />
      </div>
    </div>
  )
}
