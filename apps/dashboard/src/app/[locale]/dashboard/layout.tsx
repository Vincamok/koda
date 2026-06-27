import type { ReactNode } from 'react'
import { redirect } from 'next/navigation'
import { getSession } from '@/lib/auth'
import { AppShell } from '@/components/layout/app-shell'

interface DashboardLayoutProps {
  children: ReactNode
  params: { locale: string }
}

export default async function DashboardLayout({
  children,
  params: { locale },
}: DashboardLayoutProps) {
  const user = await getSession()

  if (!user) {
    redirect(`/${locale}/login`)
  }

  return (
    <AppShell user={user!} locale={locale}>
      {children}
    </AppShell>
  )
}
