'use client'

import * as React from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useTranslations } from 'next-intl'
import { LayoutDashboard, Boxes, Settings } from 'lucide-react'
import { cn } from '@/lib/utils'

interface BottomNavProps {
  locale: string
}

export function BottomNav({ locale }: BottomNavProps) {
  const t = useTranslations('nav')
  const pathname = usePathname()

  const items = [
    {
      href: `/${locale}/dashboard`,
      label: t('dashboard'),
      icon: <LayoutDashboard className="h-5 w-5" />,
      exact: true,
    },
    {
      href: `/${locale}/workspaces`,
      label: t('workspaces'),
      icon: <Boxes className="h-5 w-5" />,
      exact: false,
    },
    {
      href: `/${locale}/settings`,
      label: t('settings'),
      icon: <Settings className="h-5 w-5" />,
      exact: false,
    },
  ]

  return (
    <nav
      className="md:hidden fixed bottom-0 inset-x-0 z-40 bg-koda-surface border-t border-koda-border"
      style={{ paddingBottom: 'env(safe-area-inset-bottom)' }}
    >
      <div className="flex h-16 items-stretch">
        {items.map((item) => {
          const isActive = item.exact
            ? pathname === item.href
            : pathname.startsWith(item.href)

          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                'flex flex-1 flex-col items-center justify-center gap-1 py-2 min-h-[44px]',
                'text-[10px] font-medium tracking-wide transition-colors',
                isActive
                  ? 'text-koda-primary'
                  : 'text-koda-text-muted hover:text-koda-text',
              )}
            >
              {item.icon}
              <span>{item.label}</span>
            </Link>
          )
        })}
      </div>
    </nav>
  )
}
