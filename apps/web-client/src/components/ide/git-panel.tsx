'use client'

import * as React from 'react'
import { useTranslations } from 'next-intl'
import { GitBranch, Plus, Minus, Check, Upload, RefreshCw, Loader2 } from 'lucide-react'

interface GitFile {
  path: string
  status: string
}

interface GitStatus {
  branch: string
  ahead: number
  behind: number
  staged: GitFile[]
  unstaged: GitFile[]
}

interface GitPanelProps {
  workspaceId: string
}

export function GitPanel({ workspaceId }: GitPanelProps) {
  const t = useTranslations('ide')
  const [status, setStatus] = React.useState<GitStatus | null>(null)
  const [loading, setLoading] = React.useState(false)
  const [commitMsg, setCommitMsg] = React.useState('')
  const [committing, setCommitting] = React.useState(false)
  const [pushing, setPushing] = React.useState(false)

  const refresh = React.useCallback(async () => {
    setLoading(true)
    try {
      const res = await fetch(`/api/v1/workspaces/${workspaceId}/git/status`, {
        credentials: 'include',
      })
      if (res.ok) {
        const json = await res.json()
        setStatus(json.data)
      }
    } catch {
      // silently ignore
    } finally {
      setLoading(false)
    }
  }, [workspaceId])

  React.useEffect(() => {
    refresh()
  }, [refresh])

  const stage = async (path: string) => {
    await fetch(`/api/v1/workspaces/${workspaceId}/git/stage`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ paths: [path] }),
    })
    refresh()
  }

  const commit = async () => {
    if (!commitMsg.trim()) return
    setCommitting(true)
    try {
      await fetch(`/api/v1/workspaces/${workspaceId}/git/commit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ message: commitMsg }),
      })
      setCommitMsg('')
      refresh()
    } finally {
      setCommitting(false)
    }
  }

  const push = async () => {
    setPushing(true)
    try {
      await fetch(`/api/v1/workspaces/${workspaceId}/git/push`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({}),
      })
      refresh()
    } finally {
      setPushing(false)
    }
  }

  return (
    <div className="flex h-full flex-col bg-[#181825] text-[#cdd6f4]">
      {/* Header */}
      <div className="flex h-9 shrink-0 items-center justify-between border-b border-[#313244] px-3">
        <div className="flex items-center gap-2">
          <GitBranch className="h-3.5 w-3.5 text-[#89b4fa]" />
          <span className="text-xs font-medium">{t('git_panel')}</span>
          {status && (
            <span className="rounded bg-[#313244] px-1.5 py-0.5 font-mono text-[10px] text-[#6c7086]">
              {status.branch}
            </span>
          )}
        </div>
        <button
          onClick={refresh}
          disabled={loading}
          className="rounded p-1 text-[#6c7086] hover:text-[#cdd6f4] transition-colors"
        >
          <RefreshCw className={['h-3.5 w-3.5', loading ? 'animate-spin' : ''].join(' ')} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {!status ? (
          <div className="flex h-full items-center justify-center">
            <Loader2 className="h-5 w-5 animate-spin text-[#45475a]" />
          </div>
        ) : (
          <>
            {/* Staged files */}
            {status.staged.length > 0 && (
              <div className="border-b border-[#313244] py-2">
                <p className="px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-[#6c7086]">
                  {t('git_staged')} ({status.staged.length})
                </p>
                {status.staged.map((f) => (
                  <div key={f.path} className="flex items-center gap-2 px-3 py-1 hover:bg-[#313244]/40">
                    <span className="w-3 text-center text-[10px] font-bold text-emerald-400">
                      {f.status.charAt(0).toUpperCase()}
                    </span>
                    <span className="flex-1 truncate font-mono text-xs">{f.path}</span>
                    <button
                      onClick={() => stage(f.path)}
                      title={t('git_unstage')}
                      className="rounded p-0.5 text-[#6c7086] hover:text-[#f38ba8] transition-colors"
                    >
                      <Minus className="h-3 w-3" />
                    </button>
                  </div>
                ))}
              </div>
            )}

            {/* Unstaged / modified files */}
            {status.unstaged.length > 0 && (
              <div className="border-b border-[#313244] py-2">
                <p className="px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-[#6c7086]">
                  {t('git_unstaged')} ({status.unstaged.length})
                </p>
                {status.unstaged.map((f) => (
                  <div key={f.path} className="flex items-center gap-2 px-3 py-1 hover:bg-[#313244]/40">
                    <span className="w-3 text-center text-[10px] font-bold text-yellow-400">
                      {f.status.charAt(0).toUpperCase()}
                    </span>
                    <span className="flex-1 truncate font-mono text-xs">{f.path}</span>
                    <button
                      onClick={() => stage(f.path)}
                      title={t('git_stage')}
                      className="rounded p-0.5 text-[#6c7086] hover:text-[#a6e3a1] transition-colors"
                    >
                      <Plus className="h-3 w-3" />
                    </button>
                  </div>
                ))}
              </div>
            )}

            {status.staged.length === 0 && status.unstaged.length === 0 && (
              <div className="flex h-32 items-center justify-center">
                <p className="text-sm text-[#6c7086]">{t('git_no_changes')}</p>
              </div>
            )}
          </>
        )}
      </div>

      {/* Commit + push actions */}
      <div className="shrink-0 border-t border-[#313244] p-3 space-y-2">
        <textarea
          value={commitMsg}
          onChange={(e) => setCommitMsg(e.target.value)}
          placeholder={t('git_commit_message')}
          rows={2}
          className="w-full resize-none rounded border border-[#313244] bg-[#1e1e2e] px-2 py-1.5 text-xs text-[#cdd6f4] placeholder-[#45475a] focus:border-[#89b4fa] focus:outline-none transition-colors"
        />
        <div className="flex gap-2">
          <button
            onClick={commit}
            disabled={!commitMsg.trim() || committing}
            className="flex flex-1 items-center justify-center gap-1.5 rounded bg-[#89b4fa] px-3 py-1.5 text-xs font-medium text-[#1e1e2e] hover:bg-[#89b4fa]/80 disabled:cursor-not-allowed disabled:opacity-40 transition-colors"
          >
            {committing ? <Loader2 className="h-3 w-3 animate-spin" /> : <Check className="h-3 w-3" />}
            {t('git_commit')}
          </button>
          <button
            onClick={push}
            disabled={pushing}
            className="flex items-center justify-center gap-1.5 rounded border border-[#313244] px-3 py-1.5 text-xs text-[#cdd6f4] hover:bg-[#313244] disabled:cursor-not-allowed disabled:opacity-40 transition-colors"
          >
            {pushing ? <Loader2 className="h-3 w-3 animate-spin" /> : <Upload className="h-3 w-3" />}
            {t('git_push')}
          </button>
        </div>
      </div>
    </div>
  )
}
