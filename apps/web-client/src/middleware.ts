import createMiddleware from 'next-intl/middleware'
import { SUPPORTED_LOCALES, DEFAULT_LOCALE } from '@koda/i18n'

export default createMiddleware({
  locales: SUPPORTED_LOCALES,
  defaultLocale: DEFAULT_LOCALE,
  localePrefix: 'always',
})

export const config = {
  matcher: ['/((?!_next|api|.*\\..*).*)'],
}
