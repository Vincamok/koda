import { IDELayout } from '@/components/ide/ide-layout'

interface Props {
  params: { locale: string; workspaceId: string }
}

export default function WorkspacePage({ params }: Props) {
  return <IDELayout workspaceId={params.workspaceId} locale={params.locale} />
}
