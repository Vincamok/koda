'use client'

import * as React from 'react'
import {
  FileCode,
  FileText,
  Folder,
  FolderOpen,
  ChevronRight,
  Loader2,
} from 'lucide-react'

interface FileNode {
  name: string
  path: string
  type: 'file' | 'dir'
  children?: FileNode[]
}

interface FileTreeProps {
  workspaceId: string
  onFileSelect: (path: string, content: string, language: string) => void
}

const LANGUAGE_MAP: Record<string, string> = {
  rs: 'rust',
  ts: 'typescript',
  tsx: 'typescriptreact',
  js: 'javascript',
  jsx: 'javascriptreact',
  py: 'python',
  go: 'go',
  sql: 'sql',
  md: 'markdown',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'toml',
  sh: 'shell',
  bash: 'shell',
  html: 'html',
  css: 'css',
  scss: 'scss',
}

function getLanguage(filename: string): string {
  const ext = filename.split('.').pop() ?? ''
  return LANGUAGE_MAP[ext] ?? 'plaintext'
}

function FileIcon({ name }: { name: string }) {
  const ext = name.split('.').pop() ?? ''
  if (['ts', 'tsx', 'js', 'jsx', 'rs', 'py', 'go'].includes(ext)) {
    return <FileCode className="h-3.5 w-3.5 shrink-0 text-[#89b4fa]" />
  }
  return <FileText className="h-3.5 w-3.5 shrink-0 text-[#6c7086]" />
}

function FileTreeNode({
  node,
  depth,
  selectedPath,
  onSelect,
}: {
  node: FileNode
  depth: number
  selectedPath: string | null
  onSelect: (path: string) => void
}) {
  const [open, setOpen] = React.useState(depth === 0)

  if (node.type === 'dir') {
    return (
      <div>
        <button
          onClick={() => setOpen((v) => !v)}
          className="flex w-full items-center gap-1 rounded px-2 py-0.5 text-xs text-[#6c7086] hover:bg-[#313244] hover:text-[#cdd6f4] transition-colors"
          style={{ paddingLeft: `${8 + depth * 12}px` }}
        >
          <ChevronRight
            className={['h-3 w-3 shrink-0 transition-transform', open ? 'rotate-90' : ''].join(' ')}
          />
          {open ? (
            <FolderOpen className="h-3.5 w-3.5 shrink-0 text-[#f9e2af]" />
          ) : (
            <Folder className="h-3.5 w-3.5 shrink-0 text-[#f9e2af]" />
          )}
          <span className="truncate">{node.name}</span>
        </button>
        {open &&
          node.children?.map((child) => (
            <FileTreeNode
              key={child.path}
              node={child}
              depth={depth + 1}
              selectedPath={selectedPath}
              onSelect={onSelect}
            />
          ))}
      </div>
    )
  }

  return (
    <button
      onClick={() => onSelect(node.path)}
      className={[
        'flex w-full items-center gap-1.5 rounded px-2 py-0.5 text-xs transition-colors',
        selectedPath === node.path
          ? 'bg-[#313244] text-[#cdd6f4]'
          : 'text-[#6c7086] hover:bg-[#313244]/60 hover:text-[#cdd6f4]',
      ].join(' ')}
      style={{ paddingLeft: `${20 + depth * 12}px` }}
    >
      <FileIcon name={node.name} />
      <span className="truncate">{node.name}</span>
    </button>
  )
}

export function FileTree({ workspaceId, onFileSelect }: FileTreeProps) {
  const [tree, setTree] = React.useState<FileNode[]>([])
  const [loading, setLoading] = React.useState(true)
  const [selectedPath, setSelectedPath] = React.useState<string | null>(null)

  React.useEffect(() => {
    fetch(`/api/v1/workspaces/${workspaceId}/files`, { credentials: 'include' })
      .then((r) => r.json())
      .then((d) => setTree(d?.data ?? []))
      .catch(() => setTree([]))
      .finally(() => setLoading(false))
  }, [workspaceId])

  const handleSelect = async (path: string) => {
    setSelectedPath(path)
    try {
      const res = await fetch(
        `/api/v1/workspaces/${workspaceId}/files/${encodeURIComponent(path)}`,
        { credentials: 'include' },
      )
      const d = await res.json()
      const content = d?.data?.content ?? ''
      const filename = path.split('/').pop() ?? ''
      onFileSelect(path, content, getLanguage(filename))
    } catch {
      onFileSelect(path, '', 'plaintext')
    }
  }

  return (
    <div className="flex h-full flex-col bg-[#181825]">
      <div className="flex h-9 shrink-0 items-center border-b border-[#313244] px-3">
        <span className="text-xs font-medium text-[#6c7086]">FILES</span>
      </div>
      <div className="flex-1 overflow-y-auto py-1">
        {loading ? (
          <div className="flex items-center justify-center p-4">
            <Loader2 className="h-4 w-4 animate-spin text-[#6c7086]" />
          </div>
        ) : tree.length === 0 ? (
          <p className="p-3 text-xs text-[#45475a]">No files</p>
        ) : (
          tree.map((node) => (
            <FileTreeNode
              key={node.path}
              node={node}
              depth={0}
              selectedPath={selectedPath}
              onSelect={handleSelect}
            />
          ))
        )}
      </div>
    </div>
  )
}
