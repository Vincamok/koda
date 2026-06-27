import { redirect } from 'next/navigation'
import { DEFAULT_LOCALE } from '@koda/i18n'

// Redirect from root to the default locale
export default function RootPage() {
  redirect(`/${DEFAULT_LOCALE}`)
}
