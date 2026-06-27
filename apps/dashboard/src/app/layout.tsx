import type { ReactNode } from 'react'
import type { Metadata } from 'next'
import { headers } from 'next/headers'
import './globals.css'

export const metadata: Metadata = {
  title: {
    default: 'Koda',
    template: '%s | Koda',
  },
  description: 'Koda — Cloud development environments',
}

// Extracts the locale from the first path segment (e.g. /fr/... → fr).
function getLocaleFromPath(): string {
  const headersList = headers()
  const pathname = headersList.get('x-pathname') ?? '/'
  const segment = pathname.split('/')[1]
  return segment && segment.length === 2 ? segment : 'fr'
}

// Root layout: provides the html/body shell.
// The [locale] layout nests inside and wraps with NextIntlClientProvider.
export default function RootLayout({
  children,
}: {
  children: ReactNode
}) {
  const locale = getLocaleFromPath()

  return (
    <html lang={locale} suppressHydrationWarning>
      <body className="bg-koda-surface text-koda-text antialiased">
        {children}
      </body>
    </html>
  )
}
