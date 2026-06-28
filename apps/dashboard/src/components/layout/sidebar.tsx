'use client'

import * as React from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useTranslations } from 'next-intl'
import {
  LayoutDashboard,
  Boxes,
  Settings,
  X,
  Menu,
  Zap,
} from 'lucide-react'
import { cn } from '@/lib/utils'

interface SidebarProps {
  locale: string
}

interface NavItem {
  href: string
  labelKey: 'dashboard' | 'workspaces' | 'settings'
  icon: React.ReactNode
}

export function Sidebar({ locale }: SidebarProps) {
  const t = useTranslations('nav')
  const pathname = usePathname()
  const [mobileOpen, setMobileOpen] = React.useState(false)

  const navItems: NavItem[] = [
    {
      href: `/${locale}/dashboard`,
      labelKey: 'dashboard',
      icon: <LayoutDashboard className="h-4 w-4" />,
    },
    {
      href: `/${locale}/workspaces`,
      labelKey: 'workspaces',
      icon: <Boxes className="h-4 w-4" />,
    },
    {
      href: `/${locale}/settings`,
      labelKey: 'settings',
      icon: <Settings className="h-4 w-4" />,
    },
  ]

  const SidebarContent = () => (
    <aside className="flex h-full flex-col bg-koda-surface border-r border-koda-border w-60">
      {/* Logo */}
      <div className="flex h-14 items-center gap-2.5 px-5 border-b border-koda-border">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-koda-primary">
          <Zap className="h-4 w-4 text-white" />
        </div>
        <span className="font-semibold text-koda-text tracking-tight">Koda</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-3 space-y-0.5">
        {navItems.map((item) => {
          const isActive =
            pathname === item.href ||
            (item.href !== `/${locale}/dashboard` &&
              pathname.startsWith(item.href))

          return (
            <Link
              key={item.href}
              href={item.href}
              onClick={() => setMobileOpen(false)}
              className={cn(
                'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors min-h-[44px]',
                isActive
                  ? 'bg-koda-primary/15 text-koda-primary font-medium'
                  : 'text-koda-text-muted hover:bg-koda-surface-raised hover:text-koda-text',
              )}
            >
              {item.icon}
              {t(item.labelKey)}
            </Link>
          )
        })}
      </nav>

      {/* Footer */}
      <div className="p-3 border-t border-koda-border">
        <p className="text-xs text-koda-text-muted px-3">Koda v0.4.0</p>
      </div>
    </aside>
  )

  return (
    <>
      {/* Desktop sidebar */}
      <div className="hidden md:flex md:shrink-0">
        <SidebarContent />
      </div>

      {/* Mobile hamburger button */}
      <button
        className="md:hidden fixed top-3.5 left-4 z-50 rounded-md p-1.5 text-koda-text-muted hover:text-koda-text hover:bg-koda-surface-raised transition-colors"
        onClick={() => setMobileOpen(true)}
        aria-label="Open menu"
      >
        <Menu className="h-5 w-5" />
      </button>

      {/* Mobile overlay + slide-in drawer */}
      {mobileOpen && (
        <>
          <div
            className="md:hidden fixed inset-0 z-40 bg-black/60 backdrop-blur-sm"
            onClick={() => setMobileOpen(false)}
          />
          <div className="md:hidden fixed inset-y-0 left-0 z-50 flex">
            <div className="relative shadow-2xl">
              <button
                className="absolute top-3.5 right-3 rounded-md p-1.5 text-koda-text-muted hover:text-koda-text hover:bg-koda-surface-raised transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                onClick={() => setMobileOpen(false)}
                aria-label="Close menu"
              >
                <X className="h-5 w-5" />
              </button>
              <SidebarContent />
            </div>
          </div>
        </>
      )}
    </>
  )
}
