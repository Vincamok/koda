import { getRequestConfig } from 'next-intl/server'
import { notFound } from 'next/navigation'
import { SUPPORTED_LOCALES, type Locale } from '@koda/i18n'

export default getRequestConfig(async ({ locale }) => {
  if (!SUPPORTED_LOCALES.includes(locale as Locale)) {
    notFound()
  }

  return {
    messages: (await import(`../messages/${locale}.json`)).default,
  }
})
