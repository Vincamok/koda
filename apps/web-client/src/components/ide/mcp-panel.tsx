'use client'

import * as React from 'react'
import { useTranslations } from 'next-intl'
import { Plug, Plus, Trash2, ToggleLeft, ToggleRight, Loader2 } from 'lucide-react'

interface McpConnector {
  id: string
  slug: string
  name: string
  description: string | null
  category: string
}

interface McpBinding {
  id: string
  connector_slug: string
  connector_name: string
  enabled: boolean
}

interface McpPanelProps {
  workspaceId: string
}

export function McpPanel({ workspaceId }: McpPanelProps) {
  const t = useTranslations('ide')
  const [connectors, setConnectors] = React.useState<McpConnector[]>([])
  const [bindings, setBindings] = React.useState<McpBinding[]>([])
  const [loading, setLoading] = React.useState(true)
  const [adding, setAdding] = React.useState<string | null>(null)

  const base = React.useMemo(
    () => `/api/v1/workspaces/${workspaceId}/mcp`,
    [workspaceId],
  )

  const load = React.useCallback(async () => {
    setLoading(true)
    try {
      const [cr, br] = await Promise.all([
        fetch(`${base}/connectors`, { credentials: 'include' }),
        fetch(`${base}/bindings`, { credentials: 'include' }),
      ])
      if (cr.ok) setConnectors((await cr.json()).data ?? [])
      if (br.ok) setBindings((await br.json()).data ?? [])
    } finally {
      setLoading(false)
    }
  }, [base])

  React.useEffect(() => {
    load()
  }, [load])

  const boundIds = new Set(bindings.map((b) => b.connector_slug))

  const add = async (connector: McpConnector) => {
    setAdding(connector.id)
    try {
      await fetch(`${base}/bindings`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ connector_definition_id: connector.id }),
      })
      await load()
    } finally {
      setAdding(null)
    }
  }

  const remove = async (binding: McpBinding) => {
    await fetch(`${base}/bindings/${binding.id}`, {
      method: 'DELETE',
      credentials: 'include',
    })
    await load()
  }

  const toggle = async (binding: McpBinding) => {
    await fetch(`${base}/bindings`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({
        connector_definition_id: binding.id,
        enabled: !binding.enabled,
      }),
    })
    await load()
  }

  return (
    <div className="flex h-full flex-col bg-[#181825] text-[#cdd6f4]">
      {/* Header */}
      <div className="flex h-9 shrink-0 items-center gap-2 border-b border-[#313244] px-3">
        <Plug className="h-3.5 w-3.5 text-[#cba6f7]" />
        <span className="text-xs font-medium">{t('mcp_title')}</span>
      </div>

      {loading ? (
        <div className="flex h-full items-center justify-center">
          <Loader2 className="h-5 w-5 animate-spin text-[#45475a]" />
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto">
          {/* Active bindings */}
          {bindings.length > 0 && (
            <div className="border-b border-[#313244] py-2">
              <p className="px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-[#6c7086]">
                {t('mcp_enabled')} ({bindings.length})
              </p>
              {bindings.map((b) => (
                <div key={b.id} className="flex items-center gap-2 px-3 py-1.5 hover:bg-[#313244]/40">
                  <span className="flex-1 text-xs">{b.connector_name}</span>
                  <button
                    onClick={() => toggle(b)}
                    title={b.enabled ? t('mcp_disabled') : t('mcp_enabled')}
                    className="text-[#6c7086] hover:text-[#cdd6f4] transition-colors"
                  >
                    {b.enabled ? (
                      <ToggleRight className="h-4 w-4 text-[#a6e3a1]" />
                    ) : (
                      <ToggleLeft className="h-4 w-4" />
                    )}
                  </button>
                  <button
                    onClick={() => remove(b)}
                    title={t('mcp_remove')}
                    className="rounded p-0.5 text-[#6c7086] hover:text-[#f38ba8] transition-colors"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Available connectors */}
          <div className="py-2">
            <p className="px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-[#6c7086]">
              {t('mcp_add')}
            </p>
            {connectors
              .filter((c) => !boundIds.has(c.slug))
              .map((c) => (
                <div key={c.id} className="flex items-center gap-2 px-3 py-1.5 hover:bg-[#313244]/40">
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-medium truncate">{c.name}</p>
                    {c.description && (
                      <p className="text-[10px] text-[#6c7086] truncate">{c.description}</p>
                    )}
                  </div>
                  <button
                    onClick={() => add(c)}
                    disabled={adding === c.id}
                    className="shrink-0 rounded bg-[#313244] px-2 py-1 text-[10px] text-[#cdd6f4] hover:bg-[#45475a] disabled:opacity-50 transition-colors"
                  >
                    {adding === c.id ? (
                      <Loader2 className="h-3 w-3 animate-spin" />
                    ) : (
                      <Plus className="h-3 w-3" />
                    )}
                  </button>
                </div>
              ))}

            {connectors.filter((c) => !boundIds.has(c.slug)).length === 0 &&
              bindings.length === 0 && (
                <div className="flex h-24 items-center justify-center">
                  <p className="text-sm text-[#6c7086]">{t('mcp_no_bindings')}</p>
                </div>
              )}
          </div>
        </div>
      )}
    </div>
  )
}
