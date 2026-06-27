'use client'

import * as React from 'react'
import { useTranslations } from 'next-intl'
import { LogOut, User as UserIcon, ChevronDown } from 'lucide-react'
import type { User } from '@koda/shared-types'
import { logout } from '@/lib/auth'
import { cn } from '@/lib/utils'

interface HeaderProps {
  user: User
  locale: string
  title?: string
}

export function Header({ user, locale, title }: HeaderProps) {
  const t = useTranslations('auth')
  const [menuOpen, setMenuOpen] = React.useState(false)
  const menuRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleLogout = async () => {
    await logout(locale)
  }

  const initials = user.display_name
    .split(' ')
    .slice(0, 2)
    .map((n) => n[0])
    .join('')
    .toUpperCase()

  return (
    <header className="flex h-14 items-center justify-between border-b border-koda-border bg-koda-surface px-4 md:px-6">
      {/* Page title — offset on mobile to avoid hamburger overlap */}
      <div className="ml-10 md:ml-0">
        {title && (
          <h1 className="text-base font-semibold text-koda-text">{title}</h1>
        )}
      </div>

      {/* User menu */}
      <div className="relative" ref={menuRef}>
        <button
          onClick={() => setMenuOpen((v) => !v)}
          className={cn(
            'flex items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors',
            'text-koda-text hover:bg-koda-surface-raised',
          )}
          aria-haspopup="true"
          aria-expanded={menuOpen}
        >
          {user.avatar_url ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              src={user.avatar_url}
              alt={user.display_name}
              className="h-7 w-7 rounded-full object-cover"
            />
          ) : (
            <span className="flex h-7 w-7 items-center justify-center rounded-full bg-koda-primary text-xs font-medium text-white">
              {initials}
            </span>
          )}
          <span className="hidden sm:block max-w-[160px] truncate">
            {user.display_name}
          </span>
          <ChevronDown
            className={cn(
              'h-4 w-4 text-koda-text-muted transition-transform',
              menuOpen && 'rotate-180',
            )}
          />
        </button>

        {menuOpen && (
          <div className="absolute right-0 mt-1 w-48 rounded-lg border border-koda-border bg-koda-surface-raised shadow-xl py-1 z-50">
            {/* User info */}
            <div className="px-3 py-2 border-b border-koda-border">
              <p className="text-sm font-medium text-koda-text truncate">
                {user.display_name}
              </p>
              <p className="text-xs text-koda-text-muted truncate">{user.email}</p>
            </div>

            {/* Profile link */}
            <a
              href={`/${locale}/settings`}
              className="flex items-center gap-2 px-3 py-2 text-sm text-koda-text hover:bg-koda-border/40 transition-colors"
              onClick={() => setMenuOpen(false)}
            >
              <UserIcon className="h-4 w-4 text-koda-text-muted" />
              Profile
            </a>

            {/* Logout */}
            <button
              onClick={handleLogout}
              className="flex w-full items-center gap-2 px-3 py-2 text-sm text-koda-text hover:bg-koda-border/40 transition-colors"
            >
              <LogOut className="h-4 w-4 text-koda-text-muted" />
              {t('logout')}
            </button>
          </div>
        )}
      </div>
    </header>
  )
}
