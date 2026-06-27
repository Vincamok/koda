'use client'

import * as React from 'react'
import { useTranslations } from 'next-intl'
import type { WorkspaceStatus } from '@koda/shared-types'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface WorkspaceStatusProps {
  status: WorkspaceStatus
  showLabel?: boolean
}

const ANIMATED_STATUSES: WorkspaceStatus[] = [
  'cloning',
  'starting',
  'stopping',
]

const STATUS_DOT_COLORS: Record<WorkspaceStatus, string> = {
  created: 'bg-slate-400',
  cloning: 'bg-blue-400',
  ready: 'bg-indigo-400',
  starting: 'bg-amber-400',
  running: 'bg-green-400',
  stopping: 'bg-orange-400',
  stopped: 'bg-slate-400',
  reviewing: 'bg-purple-400',
  closed: 'bg-zinc-500',
  failed: 'bg-red-400',
}

const STATUS_BADGE_VARIANTS: Record<
  WorkspaceStatus,
  'created' | 'cloning' | 'ready' | 'starting' | 'running' | 'stopping' | 'stopped' | 'reviewing' | 'closed' | 'failed'
> = {
  created: 'created',
  cloning: 'cloning',
  ready: 'ready',
  starting: 'starting',
  running: 'running',
  stopping: 'stopping',
  stopped: 'stopped',
  reviewing: 'reviewing',
  closed: 'closed',
  failed: 'failed',
}

export function WorkspaceStatusBadge({
  status,
  showLabel = true,
}: WorkspaceStatusProps) {
  const t = useTranslations('workspace.status')
  const isAnimated = ANIMATED_STATUSES.includes(status)
  const dotColor = STATUS_DOT_COLORS[status]
  const badgeVariant = STATUS_BADGE_VARIANTS[status]

  return (
    <Badge variant={badgeVariant}>
      <span className="relative flex h-2 w-2">
        {isAnimated && (
          <span
            className={cn(
              'absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping',
              dotColor,
            )}
          />
        )}
        <span
          className={cn('relative inline-flex h-2 w-2 rounded-full', dotColor)}
        />
      </span>
      {showLabel && t(status)}
    </Badge>
  )
}
