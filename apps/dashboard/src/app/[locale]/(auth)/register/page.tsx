'use client'

import * as React from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useTranslations } from 'next-intl'
import { Zap } from 'lucide-react'
import { register } from '@/lib/api-client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useToast } from '@/components/ui/toast'

interface RegisterPageProps {
  params: { locale: string }
}

export default function RegisterPage({ params: { locale } }: RegisterPageProps) {
  const t = useTranslations('auth')
  const router = useRouter()
  const { addToast } = useToast()

  const [displayName, setDisplayName] = React.useState('')
  const [email, setEmail] = React.useState('')
  const [password, setPassword] = React.useState('')
  const [loading, setLoading] = React.useState(false)
  const [errors, setErrors] = React.useState<{
    displayName?: string
    email?: string
    password?: string
  }>({})

  const validate = (): boolean => {
    const next: typeof errors = {}
    if (!displayName.trim()) next.displayName = 'Nom requis'
    if (!email) next.email = 'Email requis'
    else if (!/\S+@\S+\.\S+/.test(email)) next.email = 'Email invalide'
    if (!password) next.password = 'Mot de passe requis'
    else if (password.length < 8) next.password = '8 caractères minimum'
    setErrors(next)
    return Object.keys(next).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validate()) return

    setLoading(true)
    try {
      await register(email, password, displayName)
      router.push(`/${locale}/dashboard`)
      router.refresh()
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Erreur lors de la création du compte'
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
            <h1 className="text-xl font-semibold text-koda-text">
              {t('register_title')}
            </h1>
            <p className="mt-1 text-sm text-koda-text-muted">
              {t('register_subtitle')}
            </p>
          </div>
        </div>

        {/* Form */}
        <form
          onSubmit={handleSubmit}
          className="rounded-xl border border-koda-border bg-koda-surface-raised p-6 space-y-4"
        >
          {/* Display name */}
          <div className="space-y-1.5">
            <Label htmlFor="display-name">{t('display_name')}</Label>
            <Input
              id="display-name"
              type="text"
              autoComplete="name"
              placeholder="Jean Dupont"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              error={errors.displayName}
              disabled={loading}
            />
          </div>

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
            <Label htmlFor="password">{t('password')}</Label>
            <Input
              id="password"
              type="password"
              autoComplete="new-password"
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
            {loading ? t('registering') : t('register')}
          </Button>
        </form>

        {/* Login link */}
        <p className="mt-4 text-center text-sm text-koda-text-muted">
          {t('already_account')}{' '}
          <Link
            href={`/${locale}/login`}
            className="text-koda-primary hover:underline font-medium"
          >
            {t('login')}
          </Link>
        </p>
      </div>
    </div>
  )
}
