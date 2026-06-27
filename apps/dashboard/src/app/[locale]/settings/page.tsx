import { redirect } from 'next/navigation'
import { getTranslations } from 'next-intl/server'
import { getSession } from '@/lib/auth'
import { AppShell } from '@/components/layout/app-shell'
import { SettingsForm } from './settings-form'

interface SettingsPageProps {
  params: { locale: string }
}

export default async function SettingsPage({
  params: { locale },
}: SettingsPageProps) {
  const t = await getTranslations('settings')
  const user = await getSession()

  if (!user) {
    redirect(`/${locale}/login`)
  }

  return (
    <AppShell user={user!} locale={locale} title={t('title')}>
      <SettingsForm locale={locale} initialUser={user!} />
    </AppShell>
  )
}
