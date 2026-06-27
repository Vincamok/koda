import { redirect } from 'next/navigation'
import { getSession } from '@/lib/auth'

interface LocaleRootPageProps {
  params: { locale: string }
}

export default async function LocaleRootPage({
  params: { locale },
}: LocaleRootPageProps) {
  const user = await getSession()

  if (user) {
    redirect(`/${locale}/dashboard`)
  } else {
    redirect(`/${locale}/login`)
  }
}
