import type { ReactNode } from 'react'
import type { Metadata } from 'next'
import { NextIntlClientProvider } from 'next-intl'
import { getMessages, getTranslations } from 'next-intl/server'
import { notFound } from 'next/navigation'
import { SUPPORTED_LOCALES, type Locale } from '@koda/i18n'
import { ToastProvider } from '@/components/ui/toast'

interface LocaleLayoutProps {
  children: ReactNode
  params: { locale: string }
}

export async function generateStaticParams() {
  return SUPPORTED_LOCALES.map((locale) => ({ locale }))
}

export async function generateMetadata({
  params: { locale },
}: {
  params: { locale: string }
}): Promise<Metadata> {
  const t = await getTranslations({ locale, namespace: 'nav' })
  return {
    title: {
      default: 'Koda',
      template: `%s | Koda`,
    },
    description: t('dashboard'),
  }
}

// Locale layout: wraps all pages in a locale with NextIntlClientProvider.
// Sits inside the root layout — does NOT emit <html>/<body>.
export default async function LocaleLayout({
  children,
  params: { locale },
}: LocaleLayoutProps) {
  if (!SUPPORTED_LOCALES.includes(locale as Locale)) {
    notFound()
  }

  const messages = await getMessages()

  return (
    <NextIntlClientProvider locale={locale} messages={messages}>
      <ToastProvider>{children}</ToastProvider>
    </NextIntlClientProvider>
  )
}
