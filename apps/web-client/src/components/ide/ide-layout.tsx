'use client'

import * as React from 'react'
import { PanelGroup, Panel, PanelResizeHandle } from 'react-resizable-panels'
import { useTranslations } from 'next-intl'
import { Bot, GitBranch, Plug, FileCode2, Monitor } from 'lucide-react'
import { CodeEditor } from './code-editor'
import { Terminal } from './terminal'
import { AiChat } from './ai-chat'
import { FileTree } from './file-tree'
import { GitPanel } from './git-panel'
import { McpPanel } from './mcp-panel'
import { EditorTopBar } from './editor-topbar'

interface IDELayoutProps {
  workspaceId: string
  locale: string
}

type RightTab = 'ai' | 'git' | 'mcp'
type MobileView = 'editor' | 'files' | 'ai' | 'git'

function useIsMobile(): boolean {
  const [isMobile, setIsMobile] = React.useState(false)
  React.useEffect(() => {
    const mq = window.matchMedia('(max-width: 767px)')
    setIsMobile(mq.matches)
    const handler = (e: MediaQueryListEvent) => setIsMobile(e.matches)
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])
  return isMobile
}

function useIsTablet(): boolean {
  const [isTablet, setIsTablet] = React.useState(false)
  React.useEffect(() => {
    const mq = window.matchMedia('(min-width: 768px) and (max-width: 1023px)')
    setIsTablet(mq.matches)
    const handler = (e: MediaQueryListEvent) => setIsTablet(e.matches)
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])
  return isTablet
}

export function IDELayout({ workspaceId, locale }: IDELayoutProps) {
  const t = useTranslations('ide')
  const isMobile = useIsMobile()
  const isTablet = useIsTablet()

  const [activeFile, setActiveFile] = React.useState<string | null>(null)
  const [fileContent, setFileContent] = React.useState('')
  const [language, setLanguage] = React.useState('plaintext')
  const [rightTab, setRightTab] = React.useState<RightTab>('ai')
  const [mobileView, setMobileView] = React.useState<MobileView>('editor')

  const rightPanel = (
    <div className="flex h-full flex-col">
      {/* Right panel tab bar */}
      <div className="flex h-9 shrink-0 border-b border-[#313244] bg-[#181825]">
        {(
          [
            { key: 'ai', icon: <Bot className="h-3.5 w-3.5" />, label: t('ai_chat') },
            { key: 'git', icon: <GitBranch className="h-3.5 w-3.5" />, label: t('git_panel') },
            { key: 'mcp', icon: <Plug className="h-3.5 w-3.5" />, label: t('mcp_panel') },
          ] as Array<{ key: RightTab; icon: React.ReactNode; label: string }>
        ).map((tab) => (
          <button
            key={tab.key}
            onClick={() => setRightTab(tab.key)}
            className={[
              'flex items-center gap-1.5 border-b-2 px-3 py-1.5 text-xs transition-colors',
              rightTab === tab.key
                ? 'border-[#89b4fa] text-[#89b4fa]'
                : 'border-transparent text-[#6c7086] hover:text-[#cdd6f4]',
            ].join(' ')}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>
      <div className="min-h-0 flex-1">
        {rightTab === 'ai' && (
          <AiChat workspaceId={workspaceId} currentFile={activeFile} currentContent={fileContent} />
        )}
        {rightTab === 'git' && <GitPanel workspaceId={workspaceId} />}
        {rightTab === 'mcp' && <McpPanel workspaceId={workspaceId} />}
      </div>
    </div>
  )

  // ── Mobile layout (< 768px) ────────────────────────────────────────────────
  if (isMobile) {
    return (
      <div className="flex h-[100dvh] flex-col bg-[#1e1e2e]">
        <EditorTopBar
          workspaceId={workspaceId}
          locale={locale}
          onToggleAi={() => setMobileView((v) => (v === 'ai' ? 'editor' : 'ai'))}
          aiOpen={mobileView === 'ai'}
        />

        <div className="min-h-0 flex-1">
          {mobileView === 'editor' && (
            <PanelGroup direction="vertical">
              <Panel defaultSize={65} minSize={30}>
                <CodeEditor path={activeFile} value={fileContent} language={language} onChange={setFileContent} />
              </Panel>
              <PanelResizeHandle className="h-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
              <Panel defaultSize={35} minSize={15}>
                <Terminal workspaceId={workspaceId} />
              </Panel>
            </PanelGroup>
          )}
          {mobileView === 'files' && (
            <FileTree
              workspaceId={workspaceId}
              onFileSelect={(path, content, lang) => {
                setActiveFile(path)
                setFileContent(content)
                setLanguage(lang)
                setMobileView('editor')
              }}
            />
          )}
          {mobileView === 'ai' && (
            <AiChat workspaceId={workspaceId} currentFile={activeFile} currentContent={fileContent} />
          )}
          {mobileView === 'git' && <GitPanel workspaceId={workspaceId} />}
        </div>

        {/* Mobile bottom tab bar */}
        <nav className="flex h-12 shrink-0 border-t border-[#313244] bg-[#181825]"
          style={{ paddingBottom: 'env(safe-area-inset-bottom)' }}>
          {(
            [
              { key: 'editor', icon: <Monitor className="h-4 w-4" />, label: t('mobile_editor') },
              { key: 'files', icon: <FileCode2 className="h-4 w-4" />, label: t('mobile_files') },
              { key: 'ai', icon: <Bot className="h-4 w-4" />, label: t('mobile_ai') },
              { key: 'git', icon: <GitBranch className="h-4 w-4" />, label: t('mobile_git') },
            ] as Array<{ key: MobileView; icon: React.ReactNode; label: string }>
          ).map((tab) => (
            <button
              key={tab.key}
              onClick={() => setMobileView(tab.key)}
              className={[
                'flex flex-1 flex-col items-center justify-center gap-0.5 text-[10px] transition-colors min-h-[44px]',
                mobileView === tab.key ? 'text-[#89b4fa]' : 'text-[#6c7086] hover:text-[#cdd6f4]',
              ].join(' ')}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </nav>
      </div>
    )
  }

  // ── Tablet layout (768–1023px) ─────────────────────────────────────────────
  if (isTablet) {
    return (
      <div className="flex h-[100dvh] flex-col bg-[#1e1e2e]">
        <EditorTopBar
          workspaceId={workspaceId}
          locale={locale}
          onToggleAi={() => setRightTab((t) => (t === 'ai' ? 'git' : 'ai'))}
          aiOpen={rightTab === 'ai'}
        />
        <div className="flex min-h-0 flex-1">
          <PanelGroup direction="horizontal" className="flex-1">
            <Panel defaultSize={22} minSize={15} maxSize={35}>
              <FileTree
                workspaceId={workspaceId}
                onFileSelect={(path, content, lang) => {
                  setActiveFile(path)
                  setFileContent(content)
                  setLanguage(lang)
                }}
              />
            </Panel>
            <PanelResizeHandle className="w-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
            <Panel minSize={40}>
              <PanelGroup direction="vertical">
                <Panel defaultSize={65} minSize={30}>
                  <CodeEditor path={activeFile} value={fileContent} language={language} onChange={setFileContent} />
                </Panel>
                <PanelResizeHandle className="h-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
                <Panel defaultSize={35} minSize={15}>
                  <Terminal workspaceId={workspaceId} />
                </Panel>
              </PanelGroup>
            </Panel>
            <PanelResizeHandle className="w-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
            <Panel defaultSize={30} minSize={20} maxSize={45}>
              {rightPanel}
            </Panel>
          </PanelGroup>
        </div>
      </div>
    )
  }

  // ── Desktop layout (≥ 1024px) ─────────────────────────────────────────────
  return (
    <div className="flex h-[100dvh] flex-col bg-[#1e1e2e]">
      <EditorTopBar
        workspaceId={workspaceId}
        locale={locale}
        onToggleAi={() => setRightTab((t) => (t === 'ai' ? 'git' : 'ai'))}
        aiOpen={rightTab === 'ai'}
      />

      <div className="flex min-h-0 flex-1">
        <PanelGroup direction="horizontal" className="flex-1">
          {/* File tree */}
          <Panel defaultSize={18} minSize={12} maxSize={35}>
            <FileTree
              workspaceId={workspaceId}
              onFileSelect={(path, content, lang) => {
                setActiveFile(path)
                setFileContent(content)
                setLanguage(lang)
              }}
            />
          </Panel>

          <PanelResizeHandle className="w-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />

          {/* Editor + Terminal */}
          <Panel minSize={30}>
            <PanelGroup direction="vertical">
              <Panel defaultSize={65} minSize={30}>
                <CodeEditor
                  path={activeFile}
                  value={fileContent}
                  language={language}
                  onChange={setFileContent}
                />
              </Panel>
              <PanelResizeHandle className="h-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
              <Panel defaultSize={35} minSize={15}>
                <Terminal workspaceId={workspaceId} />
              </Panel>
            </PanelGroup>
          </Panel>

          {/* Right panel: AI / Git / MCP */}
          <PanelResizeHandle className="w-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
          <Panel defaultSize={28} minSize={20} maxSize={45}>
            {rightPanel}
          </Panel>
        </PanelGroup>
      </div>
    </div>
  )
}
