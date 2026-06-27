import Link from 'next/link'
import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'
import { getAdminStats } from '@/lib/admin-api'
import type { AdminStats } from '@/lib/admin-api'

interface Props {
  params: { locale: string }
}

function InfraCard({ label, value, detail }: { label: string; value: number | string; detail?: string }) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
      <p className="text-xs font-medium uppercase tracking-wide text-zinc-500">{label}</p>
      <p className="mt-2 text-3xl font-bold text-zinc-100">{value}</p>
      {detail && <p className="mt-1 text-xs text-zinc-500">{detail}</p>}
    </div>
  )
}

export default async function InfraPage({ params: { locale } }: Props) {
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
      <div className="mb-6">
        <Link href={`/${locale}/dashboard`} className="text-sm text-zinc-400 hover:text-zinc-100">
          ← Dashboard
        </Link>
        <h1 className="mt-2 text-2xl font-bold text-zinc-100">Infrastructure</h1>
      </div>

      {stats ? (
        <>
          <section className="mb-8">
            <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-zinc-500">
              Workspaces
            </h2>
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              <InfraCard
                label="Total workspaces"
                value={stats.total_workspaces}
              />
              <InfraCard
                label="En cours d'exécution"
                value={stats.running_workspaces}
                detail="Conteneurs Docker actifs"
              />
              <InfraCard
                label="Organisations actives"
                value={stats.active_orgs}
                detail={`sur ${stats.total_orgs} au total`}
              />
              <InfraCard
                label="Pipelines"
                value={stats.total_pipelines}
              />
            </div>
          </section>

          <section>
            <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-zinc-500">
              Utilisateurs
            </h2>
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              <InfraCard
                label="Utilisateurs enregistrés"
                value={stats.total_users}
              />
            </div>
          </section>
        </>
      ) : (
        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-8 text-center text-sm text-zinc-500">
          Impossible de charger les statistiques d&apos;infrastructure — API non disponible.
        </div>
      )}
    </div>
  )
}
