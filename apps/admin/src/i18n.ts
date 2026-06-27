import { getRequestConfig } from 'next-intl/server'
import { SUPPORTED_LOCALES, type Locale } from '@koda/i18n'

export default getRequestConfig(async ({ locale }) => {
  if (!SUPPORTED_LOCALES.includes(locale as Locale)) {
    const { notFound } = await import('next/navigation')
    notFound()
  }

  const messages = (await import(`@koda/i18n/messages/${locale}.json`)).default

  return { messages }
})
