import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'
import createIntlMiddleware from 'next-intl/middleware'
import { SUPPORTED_LOCALES, DEFAULT_LOCALE } from '@koda/i18n'

const intlMiddleware = createIntlMiddleware({
  locales: SUPPORTED_LOCALES,
  defaultLocale: DEFAULT_LOCALE,
  localePrefix: 'always',
})

export default function middleware(request: NextRequest) {
  const response = intlMiddleware(request)

  // Forward the request pathname as a header so the root layout can read
  // the locale and set the html lang attribute.
  const res = response ?? NextResponse.next()
  res.headers.set('x-pathname', request.nextUrl.pathname)
  return res
}

export const config = {
  // Match all pathnames except for:
  // - /api routes
  // - /_next (Next.js internals)
  // - /_vercel (Vercel internals)
  // - static files (anything with a file extension)
  matcher: ['/((?!api|_next|_vercel|.*\\..*).*)'],
}
