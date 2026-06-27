import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: { default: 'Koda IDE', template: '%s | Koda IDE' },
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="fr" className="dark h-full">
      <body className="h-full">{children}</body>
    </html>
  )
}
