'use client'

import * as React from 'react'
import type { WorkspaceStatus } from '@koda/shared-types'
import { WorkspaceStatusBadge } from './workspace-status'

interface Props {
  workspaceId: string
  orgId: string
  initialStatus: WorkspaceStatus
}

export function WorkspaceStatusLive({ workspaceId, orgId, initialStatus }: Props) {
  const [status, setStatus] = React.useState<WorkspaceStatus>(initialStatus)

  React.useEffect(() => {
    const url = `/api/v1/organizations/${orgId}/workspaces/${workspaceId}/events`
    const es = new EventSource(url, { withCredentials: true })

    es.addEventListener('status', (e: MessageEvent) => {
      try {
        const payload = JSON.parse(e.data) as { status: WorkspaceStatus }
        setStatus(payload.status)
      } catch {
        // ignore malformed events
      }
    })

    es.onerror = () => {
      // EventSource auto-reconnects — no explicit handling needed
    }

    return () => es.close()
  }, [workspaceId, orgId])

  return <WorkspaceStatusBadge status={status} />
}
