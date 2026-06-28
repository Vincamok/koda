import { notFound, redirect } from 'next/navigation'
import Link from 'next/link'
import {
  Play,
  Trash2,
  GitBranch,
  Shield,
  Webhook,
  Activity,
  CheckCircle2,
  XCircle,
  Clock,
  AlertTriangle,
  Plus,
  ArrowLeft,
  RefreshCw,
  User,
} from 'lucide-react'
import { getSession } from '@/lib/auth'
import {
  listPipelines,
  runPipeline,
  listWebhookEvents,
  listSecurityReports,
  listWorkspaceActivity,
} from '@/lib/api-client'
import { AppShell } from '@/components/layout/app-shell'
import type { Pipeline, IncomingWebhookEvent, SecurityReport, AuditEvent } from '@koda/shared-types'

interface Props {
  params: { locale: string; id: string }
  searchParams: { tab?: string; pipeline?: string }
}

function PipelineStatusBadge({ status }: { status: string }) {
  const map: Record<string, { icon: React.ReactNode; color: string; label: string }> = {
    success: { icon: <CheckCircle2 className="h-3.5 w-3.5" />, color: 'text-emerald-400', label: 'Success' },
    failed: { icon: <XCircle className="h-3.5 w-3.5" />, color: 'text-red-400', label: 'Failed' },
    running: { icon: <Activity className="h-3.5 w-3.5 animate-pulse" />, color: 'text-blue-400', label: 'Running' },
    pending: { icon: <Clock className="h-3.5 w-3.5" />, color: 'text-yellow-400', label: 'Pending' },
    cancelled: { icon: <XCircle className="h-3.5 w-3.5" />, color: 'text-koda-text-muted', label: 'Cancelled' },
  }
  const s = map[status] ?? map['pending']
  return (
    <span className={`inline-flex items-center gap-1 text-xs font-medium ${s.color}`}>
      {s.icon}
      {s.label}
    </span>
  )
}

function PipelineTypeTag({ type }: { type: string }) {
  const colors: Record<string, string> = {
    secret_scan: 'bg-red-900/40 text-red-300',
    sast: 'bg-orange-900/40 text-orange-300',
    dependency_scan: 'bg-yellow-900/40 text-yellow-300',
    build: 'bg-blue-900/40 text-blue-300',
    lint: 'bg-purple-900/40 text-purple-300',
    image_scan: 'bg-pink-900/40 text-pink-300',
  }
  return (
    <span className={`rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${colors[type] ?? 'bg-koda-surface text-koda-text-muted'}`}>
      {type.replace('_', ' ')}
    </span>
  )
}

function SeverityBadge({ severity }: { severity: string }) {
  const colors: Record<string, string> = {
    critical: 'bg-red-900/40 text-red-300 border-red-700',
    high: 'bg-orange-900/40 text-orange-300 border-orange-700',
    medium: 'bg-yellow-900/40 text-yellow-300 border-yellow-700',
    low: 'bg-blue-900/40 text-blue-300 border-blue-700',
    info: 'bg-koda-surface text-koda-text-muted border-koda-border',
  }
  return (
    <span className={`inline-block rounded border px-1.5 py-0.5 text-[10px] font-semibold uppercase ${colors[severity] ?? colors.info}`}>
      {severity}
    </span>
  )
}

function JobStatusBadge({ status }: { status: string }) {
  const map: Record<string, { color: string; label: string }> = {
    success: { color: 'text-emerald-400', label: '✓ success' },
    failed: { color: 'text-red-400', label: '✗ failed' },
    running: { color: 'text-blue-400', label: '⟳ running' },
    pending: { color: 'text-yellow-400', label: '… pending' },
  }
  const s = map[status] ?? map['pending']
  return <span className={`text-xs font-mono ${s.color}`}>{s.label}</span>
}

export default async function WorkspaceDetailPage({ params, searchParams }: Props) {
  const { locale, id } = params
  const tab = searchParams.tab ?? 'pipelines'

  const user = await getSession()
  if (!user) redirect(`/${locale}/login`)

  const orgId = 'default'

  let pipelines: Pipeline[] = []
  let webhooks: IncomingWebhookEvent[] = []
  let reports: SecurityReport[] = []
  let activity: AuditEvent[] = []

  try {
    if (tab === 'pipelines') pipelines = await listPipelines(orgId, id)
    if (tab === 'webhooks') webhooks = await listWebhookEvents(orgId, id)
    if (tab === 'security') reports = await listSecurityReports(orgId, id)
    if (tab === 'activity') activity = await listWorkspaceActivity(orgId, id)
  } catch {
    // API unavailable — show empty states
  }

  const tabs = [
    { key: 'pipelines', label: 'Pipelines', icon: <GitBranch className="h-4 w-4" /> },
    { key: 'webhooks', label: 'Webhooks', icon: <Webhook className="h-4 w-4" /> },
    { key: 'security', label: 'Sécurité', icon: <Shield className="h-4 w-4" /> },
    { key: 'diff', label: 'Diff', icon: <GitBranch className="h-4 w-4" /> },
    { key: 'activity', label: 'Activité', icon: <Activity className="h-4 w-4" /> },
  ]

  return (
    <AppShell user={user!} locale={locale} title="Workspace">
      <div className="space-y-6">
        {/* Back */}
        <Link
          href={`/${locale}/workspaces`}
          className="inline-flex items-center gap-1.5 text-sm text-koda-text-muted hover:text-koda-text transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Workspaces
        </Link>

        {/* Tab bar — scrollable horizontally on mobile */}
        <div className="flex border-b border-koda-border overflow-x-auto [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
          {tabs.map((t) => (
            <Link
              key={t.key}
              href={`/${locale}/workspaces/${id}?tab=${t.key}`}
              className={`flex shrink-0 items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
                tab === t.key
                  ? 'border-koda-accent text-koda-accent'
                  : 'border-transparent text-koda-text-muted hover:text-koda-text'
              }`}
            >
              {t.icon}
              {t.label}
            </Link>
          ))}
        </div>

        {/* ── Pipelines tab ──────────────────────────────────────────────── */}
        {tab === 'pipelines' && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-semibold text-koda-text">Pipelines CI/CD</h3>
              <Link href={`/${locale}/workspaces/${id}/pipelines/new`}>
                <button className="inline-flex items-center gap-2 rounded-lg bg-koda-accent px-3 py-2 text-sm font-medium text-white hover:bg-koda-accent/90 transition-colors">
                  <Plus className="h-4 w-4" />
                  Nouveau pipeline
                </button>
              </Link>
            </div>

            {pipelines.length === 0 ? (
              <EmptyState
                icon={<GitBranch className="h-10 w-10 text-koda-text-muted" />}
                title="Aucun pipeline"
                description="Créez un pipeline pour automatiser vos builds, scans de sécurité et déploiements."
              />
            ) : (
              <div className="divide-y divide-koda-border rounded-xl border border-koda-border overflow-hidden">
                {pipelines.map((p) => (
                  <div key={p.id} className="bg-koda-surface">
                    {/* Pipeline row */}
                    <div className="flex items-center justify-between px-4 py-3 hover:bg-koda-surface-hover transition-colors">
                      <div className="flex items-center gap-3">
                        <PipelineTypeTag type={p.pipeline_type} />
                        <span className="text-sm font-medium text-koda-text">{p.name}</span>
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="text-xs text-koda-text-muted">
                          {new Date(p.created_at).toLocaleDateString(locale)}
                        </span>
                        <Link
                          href={`/${locale}/workspaces/${id}?tab=pipelines&pipeline=${p.id}`}
                          className="inline-flex items-center gap-1 text-xs text-koda-text-muted hover:text-koda-text transition-colors"
                        >
                          <RefreshCw className="h-3 w-3" />
                          Historique
                        </Link>
                        <form
                          action={async () => {
                            'use server'
                            await runPipeline(orgId, id, p.id)
                          }}
                        >
                          <button
                            type="submit"
                            className="inline-flex items-center gap-1.5 rounded-md bg-koda-accent/10 px-2.5 py-1 text-xs font-medium text-koda-accent hover:bg-koda-accent/20 transition-colors"
                          >
                            <Play className="h-3 w-3" />
                            Run
                          </button>
                        </form>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* ── Webhooks tab ───────────────────────────────────────────────── */}
        {tab === 'webhooks' && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-koda-text">Événements Webhook</h3>
            <p className="text-xs text-koda-text-muted">
              Endpoint: <code className="rounded bg-koda-surface px-1 py-0.5 font-mono">POST /api/v1/webhooks/{id}</code>
            </p>

            {webhooks.length === 0 ? (
              <EmptyState
                icon={<Webhook className="h-10 w-10 text-koda-text-muted" />}
                title="Aucun webhook reçu"
                description="Envoyez un POST à /api/v1/webhooks/{workspace_id} pour déclencher des pipelines."
              />
            ) : (
              <div className="divide-y divide-koda-border rounded-xl border border-koda-border overflow-hidden">
                {webhooks.map((ev) => (
                  <div
                    key={ev.id}
                    className="flex items-center justify-between bg-koda-surface px-4 py-3"
                  >
                    <div className="flex items-center gap-3">
                      {ev.hmac_valid ? (
                        <CheckCircle2 className="h-4 w-4 text-emerald-400 shrink-0" />
                      ) : (
                        <AlertTriangle className="h-4 w-4 text-yellow-400 shrink-0" />
                      )}
                      <div>
                        <p className="text-sm text-koda-text">
                          {ev.hmac_valid ? 'Signature valide' : 'Signature invalide'}
                        </p>
                        {ev.source_ip && (
                          <p className="text-xs text-koda-text-muted">IP: {ev.source_ip}</p>
                        )}
                      </div>
                    </div>
                    <span className="text-xs text-koda-text-muted">
                      {new Date(ev.received_at).toLocaleString(locale)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* ── Security tab ───────────────────────────────────────────────── */}
        {tab === 'security' && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-koda-text">Rapports de sécurité</h3>

            {reports.length === 0 ? (
              <EmptyState
                icon={<Shield className="h-10 w-10 text-koda-text-muted" />}
                title="Aucun rapport"
                description="Exécutez un pipeline secret_scan, sast ou dependency_scan pour générer un rapport."
              />
            ) : (
              <div className="space-y-3">
                {reports.map((r) => (
                  <div
                    key={r.id}
                    className="rounded-xl border border-koda-border bg-koda-surface p-4"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <PipelineTypeTag type={r.scan_type} />
                        <PipelineStatusBadge status={r.status} />
                      </div>
                      <span className="text-xs text-koda-text-muted">
                        {new Date(r.created_at).toLocaleString(locale)}
                      </span>
                    </div>
                    {r.summary && (
                      <p className="text-sm text-koda-text-muted mt-1">{r.summary}</p>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* ── Diff tab ───────────────────────────────────────────────────── */}
        {tab === 'diff' && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-koda-text">Diff viewer</h3>
            <EmptyState
              icon={<GitBranch className="h-10 w-10 text-koda-text-muted" />}
              title="Diff viewer"
              description="Ouvrez le web-client pour voir le diff Git en temps réel depuis le panel Git de l'IDE."
            />
          </div>
        )}

        {/* ── Activity tab ───────────────────────────────────────────────── */}
        {tab === 'activity' && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-koda-text">Activité</h3>

            {activity.length === 0 ? (
              <EmptyState
                icon={<Activity className="h-10 w-10 text-koda-text-muted" />}
                title="Aucune activité récente"
                description="Les actions sur ce workspace (démarrage, arrêt, pipelines, snapshots) apparaîtront ici."
              />
            ) : (
              <div className="divide-y divide-koda-border rounded-xl border border-koda-border overflow-hidden">
                {activity.map((ev) => (
                  <div
                    key={ev.id}
                    className="flex items-center gap-3 bg-koda-surface px-4 py-3"
                  >
                    <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-koda-accent/10">
                      <User className="h-3.5 w-3.5 text-koda-accent" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm text-koda-text">
                        <span className="font-medium font-mono text-koda-accent">{ev.action}</span>
                        {ev.resource_type && (
                          <span className="text-koda-text-muted"> · {ev.resource_type}</span>
                        )}
                      </p>
                    </div>
                    <span className="shrink-0 text-xs text-koda-text-muted">
                      {new Date(ev.created_at).toLocaleString(locale)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </AppShell>
  )
}

function EmptyState({
  icon,
  title,
  description,
}: {
  icon: React.ReactNode
  title: string
  description: string
}) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-koda-border py-16 gap-3">
      {icon}
      <div className="text-center">
        <p className="text-sm font-medium text-koda-text">{title}</p>
        <p className="mt-1 text-xs text-koda-text-muted max-w-xs">{description}</p>
      </div>
    </div>
  )
}
