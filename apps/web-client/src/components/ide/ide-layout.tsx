'use client'

import * as React from 'react'
import { PanelGroup, Panel, PanelResizeHandle } from 'react-resizable-panels'
import { CodeEditor } from './code-editor'
import { Terminal } from './terminal'
import { AiChat } from './ai-chat'
import { FileTree } from './file-tree'
import { EditorTopBar } from './editor-topbar'

interface IDELayoutProps {
  workspaceId: string
  locale: string
}

export function IDELayout({ workspaceId, locale }: IDELayoutProps) {
  const [activeFile, setActiveFile] = React.useState<string | null>(null)
  const [fileContent, setFileContent] = React.useState('')
  const [language, setLanguage] = React.useState('plaintext')
  const [aiOpen, setAiOpen] = React.useState(true)

  return (
    <div className="flex h-full flex-col bg-[#1e1e2e]">
      <EditorTopBar
        workspaceId={workspaceId}
        locale={locale}
        onToggleAi={() => setAiOpen((v) => !v)}
        aiOpen={aiOpen}
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

          {/* AI Chat panel */}
          {aiOpen && (
            <>
              <PanelResizeHandle className="w-px bg-[#313244] hover:bg-[#89b4fa] transition-colors" />
              <Panel defaultSize={28} minSize={20} maxSize={45}>
                <AiChat
                  workspaceId={workspaceId}
                  currentFile={activeFile}
                  currentContent={fileContent}
                />
              </Panel>
            </>
          )}
        </PanelGroup>
      </div>
    </div>
  )
}
