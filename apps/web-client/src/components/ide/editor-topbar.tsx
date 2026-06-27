'use client'

import Link from 'next/link'
import { useTranslations } from 'next-intl'
import { Bot, ArrowLeft, LayoutPanelLeft } from 'lucide-react'

interface EditorTopBarProps {
  workspaceId: string
  locale: string
  aiOpen: boolean
  onToggleAi: () => void
}

export function EditorTopBar({ workspaceId, locale, aiOpen, onToggleAi }: EditorTopBarProps) {
  const t = useTranslations('ide')

  return (
    <header className="flex h-9 shrink-0 items-center justify-between border-b border-[#313244] bg-[#181825] px-3">
      <div className="flex items-center gap-3">
        <Link
          href={`/${locale}/workspaces`}
          className="flex items-center gap-1.5 text-xs text-[#6c7086] hover:text-[#cdd6f4] transition-colors"
        >
          <ArrowLeft className="h-3.5 w-3.5" />
          {t('back')}
        </Link>
        <span className="text-[#313244]">|</span>
        <span className="font-mono text-xs text-[#6c7086]">
          ws:{workspaceId.slice(0, 8)}
        </span>
      </div>

      <div className="flex items-center gap-1">
        <button
          onClick={onToggleAi}
          title={t('toggle_ai')}
          className={[
            'flex items-center gap-1.5 rounded px-2 py-1 text-xs transition-colors',
            aiOpen
              ? 'bg-[#89b4fa]/10 text-[#89b4fa]'
              : 'text-[#6c7086] hover:text-[#cdd6f4]',
          ].join(' ')}
        >
          <Bot className="h-3.5 w-3.5" />
          {t('ai_chat')}
        </button>
      </div>
    </header>
  )
}
