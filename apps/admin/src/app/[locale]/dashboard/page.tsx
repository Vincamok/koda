import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'

interface Props {
  params: { locale: string }
}

export default async function AdminDashboard({ params: { locale } }: Props) {
  const user = await getAdminSession()
  if (!user) redirect(`/${locale}/login`)

  return (
    <div className="min-h-screen bg-zinc-950 p-8">
      <header className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-zinc-100">Koda Admin</h1>
          <p className="mt-0.5 text-sm text-zinc-400">Logged in as {user.email}</p>
        </div>
        <form action={`/api/logout`} method="POST">
          <button
            type="submit"
            className="rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:border-zinc-500 hover:text-zinc-100 transition-colors"
          >
            Sign out
          </button>
        </form>
      </header>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <NavCard title="Users" description="Manage users and super-admin flags" href={`/${locale}/users`} />
        <NavCard title="Organizations" description="View and manage all organizations" href={`/${locale}/orgs`} />
        <NavCard title="Health" description="API health status" href="/api/v1/admin/health" external />
      </div>
    </div>
  )
}

function NavCard({
  title,
  description,
  href,
  external,
}: {
  title: string
  description: string
  href: string
  external?: boolean
}) {
  return (
    <a
      href={href}
      target={external ? '_blank' : undefined}
      rel={external ? 'noopener noreferrer' : undefined}
      className="group block rounded-xl border border-zinc-800 bg-zinc-900 p-6 transition-colors hover:border-indigo-800 hover:bg-zinc-800/60"
    >
      <h2 className="font-semibold text-zinc-100 group-hover:text-indigo-300 transition-colors">
        {title}
      </h2>
      <p className="mt-1 text-sm text-zinc-400">{description}</p>
    </a>
  )
}
