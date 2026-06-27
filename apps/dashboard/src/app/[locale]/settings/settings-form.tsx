'use client'

import * as React from 'react'
import { useRouter } from 'next/navigation'
import { useTranslations } from 'next-intl'
import { User as UserIcon, Globe, Palette, Save } from 'lucide-react'
import { getUserSettings, updateUserSettings } from '@/lib/api-client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { useToast } from '@/components/ui/toast'
import type { User, UserSettings } from '@koda/shared-types'

interface SettingsFormProps {
  locale: string
  initialUser: User
}

export function SettingsForm({ locale, initialUser }: SettingsFormProps) {
  const t = useTranslations('settings')
  const router = useRouter()
  const { addToast } = useToast()

  const [displayName, setDisplayName] = React.useState(initialUser.display_name)
  const [selectedLocale, setSelectedLocale] = React.useState(locale)
  const [selectedTheme, setSelectedTheme] = React.useState('default')
  const [loading, setLoading] = React.useState(true)
  const [saving, setSaving] = React.useState(false)

  React.useEffect(() => {
    getUserSettings()
      .then((s) => {
        setSelectedLocale(s.locale)
        setSelectedTheme(s.theme_id ?? 'default')
      })
      .catch(() => {
        // Use defaults if settings not available
      })
      .finally(() => setLoading(false))
  }, [])

  const handleSave = async () => {
    setSaving(true)
    try {
      await updateUserSettings({
        locale: selectedLocale as UserSettings['locale'],
        theme_id: selectedTheme,
      })
      addToast(t('saved'), 'success')

      // If locale changed, redirect to new locale URL
      if (selectedLocale !== locale) {
        router.push(`/${selectedLocale}/settings`)
        router.refresh()
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Erreur'
      addToast(message, 'error')
    } finally {
      setSaving(false)
    }
  }

  const localeOptions = [
    { value: 'fr', label: t('locale_options.fr') },
    { value: 'en', label: t('locale_options.en') },
    { value: 'es', label: t('locale_options.es') },
    { value: 'de', label: t('locale_options.de') },
  ]

  const themeOptions = [
    { value: 'default', label: t('theme_options.default') },
    { value: 'light', label: t('theme_options.light') },
  ]

  if (loading) {
    return (
      <div className="flex h-40 items-center justify-center">
        <div className="h-6 w-6 animate-spin rounded-full border-2 border-koda-primary border-t-transparent" />
      </div>
    )
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      {/* Page header */}
      <div>
        <h2 className="text-2xl font-bold text-koda-text">{t('title')}</h2>
        <p className="mt-1 text-sm text-koda-text-muted">
          Gérez votre profil et vos préférences.
        </p>
      </div>

      {/* Profile section */}
      <section className="rounded-xl border border-koda-border bg-koda-surface-raised">
        <div className="flex items-center gap-3 border-b border-koda-border px-6 py-4">
          <UserIcon className="h-5 w-5 text-koda-text-muted" />
          <h3 className="font-semibold text-koda-text">{t('profile')}</h3>
        </div>
        <div className="p-6 space-y-4">
          {/* Avatar */}
          <div className="flex items-center gap-4">
            {initialUser.avatar_url ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={initialUser.avatar_url}
                alt={initialUser.display_name}
                className="h-16 w-16 rounded-full object-cover ring-2 ring-koda-border"
              />
            ) : (
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-koda-primary text-white text-xl font-semibold ring-2 ring-koda-border">
                {initialUser.display_name
                  .split(' ')
                  .slice(0, 2)
                  .map((n) => n[0])
                  .join('')
                  .toUpperCase()}
              </div>
            )}
            <div>
              <p className="text-sm font-medium text-koda-text">
                {initialUser.display_name}
              </p>
              <p className="text-xs text-koda-text-muted">{initialUser.email}</p>
            </div>
          </div>

          {/* Display name */}
          <div className="space-y-1.5">
            <Label htmlFor="display-name">{t('display_name')}</Label>
            <Input
              id="display-name"
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              disabled={saving}
            />
          </div>

          {/* Email (read-only) */}
          <div className="space-y-1.5">
            <Label htmlFor="email">{t('email')}</Label>
            <Input
              id="email"
              type="email"
              value={initialUser.email}
              disabled
              className="opacity-60"
            />
            <p className="text-xs text-koda-text-muted">{t('email_hint')}</p>
          </div>
        </div>
      </section>

      {/* Language section */}
      <section className="rounded-xl border border-koda-border bg-koda-surface-raised">
        <div className="flex items-center gap-3 border-b border-koda-border px-6 py-4">
          <Globe className="h-5 w-5 text-koda-text-muted" />
          <h3 className="font-semibold text-koda-text">{t('locale')}</h3>
        </div>
        <div className="p-6">
          <div className="space-y-1.5">
            <Label htmlFor="locale-select">{t('locale')}</Label>
            <Select
              id="locale-select"
              options={localeOptions}
              value={selectedLocale}
              onValueChange={setSelectedLocale}
              disabled={saving}
            />
          </div>
        </div>
      </section>

      {/* Theme section */}
      <section className="rounded-xl border border-koda-border bg-koda-surface-raised">
        <div className="flex items-center gap-3 border-b border-koda-border px-6 py-4">
          <Palette className="h-5 w-5 text-koda-text-muted" />
          <h3 className="font-semibold text-koda-text">{t('theme')}</h3>
        </div>
        <div className="p-6 space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="theme-select">{t('theme')}</Label>
            <Select
              id="theme-select"
              options={themeOptions}
              value={selectedTheme}
              onValueChange={setSelectedTheme}
              disabled={saving}
            />
          </div>

          {/* Theme preview swatches */}
          <div className="flex gap-3">
            {themeOptions.map((opt) => (
              <button
                key={opt.value}
                type="button"
                onClick={() => setSelectedTheme(opt.value)}
                className={`flex flex-col items-center gap-2 rounded-lg border-2 p-3 transition-colors ${
                  selectedTheme === opt.value
                    ? 'border-koda-primary'
                    : 'border-koda-border hover:border-koda-text-muted'
                }`}
              >
                <div
                  className={`h-12 w-20 rounded-md ${
                    opt.value === 'default' ? 'bg-[#1e1e2e]' : 'bg-[#f8f8fc]'
                  }`}
                >
                  <div
                    className={`h-3 rounded-t-md ${
                      opt.value === 'default' ? 'bg-[#2a2a3e]' : 'bg-[#e8e8f0]'
                    }`}
                  />
                </div>
                <span className="text-xs text-koda-text-muted">{opt.label}</span>
              </button>
            ))}
          </div>
        </div>
      </section>

      {/* Save button */}
      <div className="flex justify-end">
        <Button
          variant="primary"
          size="md"
          onClick={handleSave}
          disabled={saving}
          className="min-w-[120px]"
        >
          <Save className="h-4 w-4" />
          {saving ? t('saving') : t('save')}
        </Button>
      </div>
    </div>
  )
}
