import type { ReactNode } from 'react'
import type { User } from '@koda/shared-types'
import { Sidebar } from './sidebar'
import { Header } from './header'

interface AppShellProps {
  user: User
  locale: string
  title?: string
  children: ReactNode
}

export function AppShell({ user, locale, title, children }: AppShellProps) {
  return (
    <div className="flex h-screen overflow-hidden bg-koda-surface">
      <Sidebar locale={locale} />
      <div className="flex flex-1 flex-col min-w-0 overflow-hidden">
        <Header user={user} locale={locale} title={title} />
        <main className="flex-1 overflow-y-auto p-6">{children}</main>
      </div>
    </div>
  )
}
