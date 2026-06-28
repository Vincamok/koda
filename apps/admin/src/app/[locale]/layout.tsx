import { NextIntlClientProvider } from 'next-intl'
import { getMessages, setRequestLocale } from 'next-intl/server'
import { notFound } from 'next/navigation'
import { SUPPORTED_LOCALES, type Locale } from '@koda/i18n'

interface LocaleLayoutProps {
  children: React.ReactNode
  params: { locale: string }
}

export const dynamic = 'force-dynamic'

export function generateStaticParams() {
  return SUPPORTED_LOCALES.map((locale) => ({ locale }))
}

export default async function LocaleLayout({ children, params: { locale } }: LocaleLayoutProps) {
  if (!SUPPORTED_LOCALES.includes(locale as Locale)) {
    notFound()
  }
  setRequestLocale(locale)
  const messages = await getMessages()
  return (
    <html lang={locale} className="dark">
      <body>
        <NextIntlClientProvider locale={locale} messages={messages}>
          {children}
        </NextIntlClientProvider>
      </body>
    </html>
  )
}
