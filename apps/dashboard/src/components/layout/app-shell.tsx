import type { ReactNode } from 'react'
import type { User } from '@koda/shared-types'
import { Sidebar } from './sidebar'
import { Header } from './header'
import { BottomNav } from './bottom-nav'

interface AppShellProps {
  user: User
  locale: string
  title?: string
  children: ReactNode
}

export function AppShell({ user, locale, title, children }: AppShellProps) {
  return (
    <div className="flex h-[100dvh] overflow-hidden bg-koda-surface">
      <Sidebar locale={locale} />
      <div className="flex flex-1 flex-col min-w-0 overflow-hidden">
        <Header user={user} locale={locale} title={title} />
        {/* pb-20 md:pb-0 clears the fixed bottom nav on mobile */}
        <main className="flex-1 overflow-y-auto p-4 sm:p-6 pb-20 md:pb-6">
          {children}
        </main>
      </div>
      <BottomNav locale={locale} />
    </div>
  )
}
