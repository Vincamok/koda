'use client'

import * as React from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useTranslations } from 'next-intl'
import { Zap } from 'lucide-react'
import { login } from '@/lib/api-client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useToast } from '@/components/ui/toast'

interface LoginPageProps {
  params: { locale: string }
}

// OAuth provider icons as SVGs
function GoogleIcon() {
  return (
    <svg viewBox="0 0 24 24" className="h-4 w-4" aria-hidden>
      <path
        d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
        fill="#4285F4"
      />
      <path
        d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
        fill="#34A853"
      />
      <path
        d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
        fill="#FBBC05"
      />
      <path
        d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
        fill="#EA4335"
      />
    </svg>
  )
}

function GitHubIcon() {
  return (
    <svg viewBox="0 0 24 24" className="h-4 w-4 fill-current" aria-hidden>
      <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
    </svg>
  )
}

function AuthentikIcon() {
  return (
    <svg viewBox="0 0 24 24" className="h-4 w-4 fill-current" aria-hidden>
      <circle cx="12" cy="12" r="10" className="opacity-20" />
      <path d="M12 2a10 10 0 1 0 0 20A10 10 0 0 0 12 2zm0 3a3 3 0 1 1 0 6 3 3 0 0 1 0-6zm0 14.2a7.2 7.2 0 0 1-6-3.22c.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08a7.2 7.2 0 0 1-6 3.22z" />
    </svg>
  )
}

export default function LoginPage({ params: { locale } }: LoginPageProps) {
  const t = useTranslations('auth')
  const router = useRouter()
  const { addToast } = useToast()

  const [email, setEmail] = React.useState('')
  const [password, setPassword] = React.useState('')
  const [loading, setLoading] = React.useState(false)
  const [errors, setErrors] = React.useState<{
    email?: string
    password?: string
  }>({})

  const validate = (): boolean => {
    const next: typeof errors = {}
    if (!email) next.email = 'Email requis'
    else if (!/\S+@\S+\.\S+/.test(email)) next.email = 'Email invalide'
    if (!password) next.password = 'Mot de passe requis'
    setErrors(next)
    return Object.keys(next).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validate()) return

    setLoading(true)
    try {
      await login(email, password)
      router.push(`/${locale}/dashboard`)
      router.refresh()
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Erreur de connexion'
      addToast(message, 'error')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-koda-surface px-4">
      <div className="w-full max-w-sm">
        {/* Logo */}
        <div className="mb-8 flex flex-col items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-koda-primary shadow-lg shadow-koda-primary/30">
            <Zap className="h-6 w-6 text-white" />
          </div>
          <div className="text-center">
            <h1 className="text-xl font-semibold text-koda-text">{t('sign_in_title')}</h1>
            <p className="mt-1 text-sm text-koda-text-muted">{t('sign_in_subtitle')}</p>
          </div>
        </div>

        {/* Form */}
        <form
          onSubmit={handleSubmit}
          className="rounded-xl border border-koda-border bg-koda-surface-raised p-6 space-y-4"
        >
          {/* Email */}
          <div className="space-y-1.5">
            <Label htmlFor="email">{t('email')}</Label>
            <Input
              id="email"
              type="email"
              autoComplete="email"
              placeholder="you@example.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              error={errors.email}
              disabled={loading}
            />
          </div>

          {/* Password */}
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
              <Label htmlFor="password">{t('password')}</Label>
              <a
                href={`/${locale}/forgot-password`}
                className="text-xs text-koda-text-muted hover:text-koda-primary transition-colors"
              >
                {t('forgot_password')}
              </a>
            </div>
            <Input
              id="password"
              type="password"
              autoComplete="current-password"
              placeholder="••••••••"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              error={errors.password}
              disabled={loading}
            />
          </div>

          {/* Submit */}
          <Button
            type="submit"
            variant="primary"
            size="md"
            className="w-full mt-2"
            disabled={loading}
          >
            {loading ? t('submitting') : t('login')}
          </Button>

          {/* Divider */}
          <div className="relative my-2">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-koda-border" />
            </div>
            <div className="relative flex justify-center text-xs">
              <span className="bg-koda-surface-raised px-2 text-koda-text-muted">
                {t('or_continue_with')}
              </span>
            </div>
          </div>

          {/* OAuth buttons */}
          <div className="grid grid-cols-3 gap-2">
            <a
              href={`${process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'}/api/v1/auth/oauth/google`}
              className="flex items-center justify-center gap-1.5 rounded-md border border-koda-border bg-koda-surface px-3 py-2 text-xs font-medium text-koda-text hover:bg-koda-border/40 transition-colors"
            >
              <GoogleIcon />
              Google
            </a>
            <a
              href={`${process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'}/api/v1/auth/oauth/github`}
              className="flex items-center justify-center gap-1.5 rounded-md border border-koda-border bg-koda-surface px-3 py-2 text-xs font-medium text-koda-text hover:bg-koda-border/40 transition-colors"
            >
              <GitHubIcon />
              GitHub
            </a>
            <a
              href={`${process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'}/api/v1/auth/oauth/authentik`}
              className="flex items-center justify-center gap-1.5 rounded-md border border-koda-border bg-koda-surface px-3 py-2 text-xs font-medium text-koda-text hover:bg-koda-border/40 transition-colors"
            >
              <AuthentikIcon />
              SSO
            </a>
          </div>
        </form>

        {/* Register link */}
        <p className="mt-4 text-center text-sm text-koda-text-muted">
          {t('no_account')}{' '}
          <Link
            href={`/${locale}/register`}
            className="text-koda-primary hover:underline font-medium"
          >
            {t('register')}
          </Link>
        </p>
      </div>
    </div>
  )
}
