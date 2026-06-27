'use client'

import * as React from 'react'
import Link from 'next/link'
import { useTranslations } from 'next-intl'
import { GitBranch, Cpu, HardDrive, ArrowRight } from 'lucide-react'
import type { Workspace } from '@koda/shared-types'
import { Card, CardContent, CardFooter } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { WorkspaceStatusLive } from './workspace-status-live'

interface WorkspaceCardProps {
  workspace: Workspace
  locale: string
  orgId: string
}

export function WorkspaceCard({ workspace, locale, orgId }: WorkspaceCardProps) {
  const t = useTranslations('workspace')

  const updatedAt = new Date(workspace.updated_at).toLocaleDateString(locale, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  })

  return (
    <Card className="group flex flex-col transition-colors hover:border-koda-primary/50">
      <CardContent className="flex-1 pt-6">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <h3 className="truncate font-semibold text-koda-text group-hover:text-koda-primary transition-colors">
              {workspace.name}
            </h3>
            <p className="mt-0.5 text-xs text-koda-text-muted">
              {t('updated_at', { date: updatedAt })}
            </p>
          </div>
          <WorkspaceStatusLive
            workspaceId={workspace.id}
            orgId={orgId}
            initialStatus={workspace.status}
          />
        </div>

        {/* Resource info */}
        <div className="mt-4 flex flex-wrap gap-3">
          <div className="flex items-center gap-1.5 text-xs text-koda-text-muted">
            <Cpu className="h-3.5 w-3.5" />
            {t('cpu', { cores: workspace.cpu_limit })}
          </div>
          <div className="flex items-center gap-1.5 text-xs text-koda-text-muted">
            <HardDrive className="h-3.5 w-3.5" />
            {t('ram', { ram: workspace.ram_limit_mb })}
          </div>
          <div className="flex items-center gap-1.5 text-xs text-koda-text-muted">
            <GitBranch className="h-3.5 w-3.5" />
            <span className="font-mono truncate max-w-[120px]">{workspace.uid}</span>
          </div>
        </div>
      </CardContent>

      <CardFooter className="border-t border-koda-border/50">
        <Link
          href={`/${locale}/workspaces/${workspace.id}`}
          className="w-full"
        >
          <Button variant="ghost" size="sm" className="w-full justify-between">
            {t('open')}
            <ArrowRight className="h-4 w-4" />
          </Button>
        </Link>
      </CardFooter>
    </Card>
  )
}
