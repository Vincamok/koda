'use client'

import * as React from 'react'
import Editor, { loader } from '@monaco-editor/react'
import type * as Monaco from 'monaco-editor'

// Use local monaco (bundled), not CDN
loader.config({ paths: { vs: '/_next/static/monaco/vs' } })

interface CodeEditorProps {
  path: string | null
  value: string
  language: string
  onChange: (value: string) => void
}

const CATPPUCCIN_THEME: Monaco.editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6c7086', fontStyle: 'italic' },
    { token: 'keyword', foreground: 'cba6f7' },
    { token: 'string', foreground: 'a6e3a1' },
    { token: 'number', foreground: 'fab387' },
    { token: 'type', foreground: 'f38ba8' },
    { token: 'function', foreground: '89b4fa' },
    { token: 'variable', foreground: 'cdd6f4' },
  ],
  colors: {
    'editor.background': '#1e1e2e',
    'editor.foreground': '#cdd6f4',
    'editor.lineHighlightBackground': '#313244',
    'editorLineNumber.foreground': '#45475a',
    'editorLineNumber.activeForeground': '#cdd6f4',
    'editor.selectionBackground': '#45475a',
    'editorCursor.foreground': '#f5c2e7',
    'editor.findMatchBackground': '#fab38740',
    'editorWidget.background': '#181825',
    'editorWidget.border': '#313244',
    'input.background': '#313244',
    'input.foreground': '#cdd6f4',
  },
}

export function CodeEditor({ path, value, language, onChange }: CodeEditorProps) {
  const monacoRef = React.useRef<typeof Monaco | null>(null)

  const handleMount = (_editor: Monaco.editor.IStandaloneCodeEditor, monaco: typeof Monaco) => {
    monacoRef.current = monaco
    monaco.editor.defineTheme('catppuccin', CATPPUCCIN_THEME)
    monaco.editor.setTheme('catppuccin')
  }

  if (!path) {
    return (
      <div className="flex h-full items-center justify-center bg-[#1e1e2e]">
        <p className="text-sm text-[#45475a]">Select a file to start editing</p>
      </div>
    )
  }

  return (
    <Editor
      height="100%"
      path={path}
      language={language}
      value={value}
      theme="catppuccin"
      onMount={handleMount}
      onChange={(v) => onChange(v ?? '')}
      options={{
        fontSize: 14,
        fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace',
        fontLigatures: true,
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        bracketPairColorization: { enabled: true },
        formatOnPaste: true,
        formatOnType: true,
        tabSize: 2,
        renderWhitespace: 'boundary',
        smoothScrolling: true,
        cursorBlinking: 'phase',
        lineNumbers: 'on',
        glyphMargin: false,
        folding: true,
        padding: { top: 12 },
      }}
    />
  )
}
