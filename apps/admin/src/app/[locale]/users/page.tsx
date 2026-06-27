import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'

interface Props {
  params: { locale: string }
}

export default async function UsersPage({ params: { locale } }: Props) {
  const user = await getAdminSession()
  if (!user) redirect(`/${locale}/login`)

  return (
    <div className="min-h-screen bg-zinc-950 p-8">
      <div className="mb-6">
        <a href={`/${locale}/dashboard`} className="text-sm text-zinc-400 hover:text-zinc-100">
          ← Dashboard
        </a>
        <h1 className="mt-2 text-2xl font-bold text-zinc-100">Users</h1>
      </div>
      <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-6 text-sm text-zinc-400">
        User management — coming soon (Phase 1)
      </div>
    </div>
  )
}
