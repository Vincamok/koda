'use client'

import * as React from 'react'
import { Save, FileText, Terminal, GitBranch, Bot, Loader2 } from 'lucide-react'
import { useTranslations } from 'next-intl'

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'

interface PersonalFile {
  path: string
  content: string
}

const FILE_LABELS: Record<string, { label: string; icon: React.ReactNode; description: string }> = {
  'ai/instructions.md': {
    label: 'Instructions IA',
    icon: <Bot className="h-3.5 w-3.5" />,
    description: 'Instructions personnelles injectées dans tous les contextes LLM (couche 6)',
  },
  'shell/bashrc': {
    label: '.bashrc',
    icon: <Terminal className="h-3.5 w-3.5" />,
    description: 'Configuration bash sourcée dans les terminaux',
  },
  'shell/zshrc': {
    label: '.zshrc',
    icon: <Terminal className="h-3.5 w-3.5" />,
    description: 'Configuration zsh sourcée dans les terminaux',
  },
  'shell/aliases': {
    label: 'Aliases',
    icon: <Terminal className="h-3.5 w-3.5" />,
    description: 'Aliases shell disponibles dans tous les workspaces',
  },
  'git/.gitconfig': {
    label: '.gitconfig',
    icon: <GitBranch className="h-3.5 w-3.5" />,
    description: 'Configuration git personnelle montée dans les containers',
  },
  'notes/personal.md': {
    label: 'Notes',
    icon: <FileText className="h-3.5 w-3.5" />,
    description: 'Notes personnelles',
  },
}

export function PersonalPanel() {
  const [files, setFiles] = React.useState<PersonalFile[]>([])
  const [selectedFile, setSelectedFile] = React.useState<string>('ai/instructions.md')
  const [content, setContent] = React.useState('')
  const [saving, setSaving] = React.useState(false)
  const [saved, setSaved] = React.useState(false)
  const [loading, setLoading] = React.useState(true)
  const [dirty, setDirty] = React.useState(false)

  React.useEffect(() => {
    setLoading(true)
    fetch(`${API_BASE}/api/v1/personal/files`, { credentials: 'include' })
      .then((r) => r.json())
      .then((json) => {
        const data: PersonalFile[] = json.data ?? []
        setFiles(data)
        const current = data.find((f) => f.path === selectedFile)
        setContent(current?.content ?? '')
        setLoading(false)
      })
      .catch(() => setLoading(false))
  }, [])

  const handleSelectFile = (path: string) => {
    if (dirty) {
      if (!window.confirm('Unsaved changes — switch anyway?')) return
    }
    setSelectedFile(path)
    const f = files.find((f) => f.path === path)
    setContent(f?.content ?? '')
    setDirty(false)
    setSaved(false)
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      const res = await fetch(`${API_BASE}/api/v1/personal/files/${selectedFile}`, {
        method: 'PUT',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      })
      if (res.ok) {
        setFiles((prev) =>
          prev.map((f) => (f.path === selectedFile ? { ...f, content } : f))
        )
        setDirty(false)
        setSaved(true)
        setTimeout(() => setSaved(false), 2000)
      }
    } finally {
      setSaving(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      handleSave()
    }
  }

  const meta = FILE_LABELS[selectedFile]

  return (
    <div className="flex h-full flex-col bg-[#1e1e2e]">
      {/* File list */}
      <div className="shrink-0 border-b border-[#313244]">
        <div className="flex flex-col gap-0.5 p-1.5">
          {Object.entries(FILE_LABELS).map(([path, { label, icon }]) => (
            <button
              key={path}
              onClick={() => handleSelectFile(path)}
              className={[
                'flex items-center gap-2 rounded px-2.5 py-1.5 text-left text-xs transition-colors',
                selectedFile === path
                  ? 'bg-[#313244] text-[#cdd6f4]'
                  : 'text-[#6c7086] hover:bg-[#24243e] hover:text-[#cdd6f4]',
              ].join(' ')}
            >
              {icon}
              <span className="font-mono">{label}</span>
              {selectedFile === path && dirty && (
                <span className="ml-auto h-1.5 w-1.5 rounded-full bg-[#f9e2af]" />
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Editor area */}
      <div className="flex min-h-0 flex-1 flex-col">
        {/* File header */}
        <div className="flex items-center justify-between border-b border-[#313244] bg-[#181825] px-3 py-1.5">
          <div className="flex flex-col">
            <span className="font-mono text-xs text-[#cdd6f4]">.personal/{selectedFile}</span>
            {meta && (
              <span className="mt-0.5 text-[10px] text-[#6c7086]">{meta.description}</span>
            )}
          </div>
          <button
            onClick={handleSave}
            disabled={saving || !dirty}
            className={[
              'flex items-center gap-1 rounded px-2 py-1 text-[10px] font-medium transition-colors',
              saved
                ? 'bg-emerald-900/30 text-emerald-400'
                : dirty
                ? 'bg-[#89b4fa]/10 text-[#89b4fa] hover:bg-[#89b4fa]/20'
                : 'cursor-default text-[#6c7086]',
            ].join(' ')}
          >
            {saving ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <Save className="h-3 w-3" />
            )}
            {saved ? 'Saved' : 'Save'}
          </button>
        </div>

        {/* Textarea editor */}
        {loading ? (
          <div className="flex flex-1 items-center justify-center">
            <Loader2 className="h-5 w-5 animate-spin text-[#6c7086]" />
          </div>
        ) : (
          <textarea
            value={content}
            onChange={(e) => {
              setContent(e.target.value)
              setDirty(true)
              setSaved(false)
            }}
            onKeyDown={handleKeyDown}
            spellCheck={false}
            className="min-h-0 flex-1 resize-none bg-[#1e1e2e] p-3 font-mono text-xs text-[#cdd6f4] outline-none placeholder:text-[#45475a]"
            placeholder={`# ${selectedFile}\n\n— vide —`}
          />
        )}
      </div>
    </div>
  )
}
