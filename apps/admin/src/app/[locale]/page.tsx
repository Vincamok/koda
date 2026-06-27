import { redirect } from 'next/navigation'
import { getAdminSession } from '@/lib/auth'

interface Props {
  params: { locale: string }
}

export default async function LocaleRootPage({ params: { locale } }: Props) {
  const user = await getAdminSession()
  if (user) {
    redirect(`/${locale}/dashboard`)
  } else {
    redirect(`/${locale}/login`)
  }
}
