import createMiddleware from 'next-intl/middleware'
import { NextRequest, NextResponse } from 'next/server'
import { SUPPORTED_LOCALES, DEFAULT_LOCALE } from '@koda/i18n'

const intlMiddleware = createMiddleware({
  locales: SUPPORTED_LOCALES,
  defaultLocale: DEFAULT_LOCALE,
  localePrefix: 'always',
})

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl

  // Skip intl for API and static files
  if (
    pathname.startsWith('/_next') ||
    pathname.startsWith('/api') ||
    pathname.includes('.')
  ) {
    return NextResponse.next()
  }

  return intlMiddleware(request)
}

export const config = {
  matcher: ['/((?!_next|api|.*\\..*).*)'],
}
