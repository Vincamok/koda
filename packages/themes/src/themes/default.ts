import type { Skin } from '../types'

export const defaultSkin: Skin = {
  id: 'default',
  name: 'Koda Default',
  description: 'Interface sombre classique. Sidebar gauche, éditeur centré, IA à droite.',
  colorMode: 'dark',
  layout: 'sidebar-left',
  density: 'comfortable',
  previewColor: '#1a1d27',

  colors: {
    '--background':           '#0f1117',
    '--background-secondary': '#1a1d27',
    '--background-tertiary':  '#222535',
    '--surface':              '#1e2130',
    '--surface-hover':        '#262b40',
    '--surface-active':       '#2d3452',

    '--foreground':           '#e2e8f0',
    '--foreground-muted':     '#94a3b8',
    '--foreground-subtle':    '#475569',

    '--primary':              '#6366f1',
    '--primary-foreground':   '#ffffff',
    '--primary-hover':        '#4f46e5',
    '--secondary':            '#334155',
    '--secondary-foreground': '#cbd5e1',

    '--success':              '#22c55e',
    '--warning':              '#f59e0b',
    '--destructive':          '#ef4444',
    '--info':                 '#3b82f6',

    '--border':               '#2d3452',
    '--border-strong':        '#4a5568',

    '--editor-background':    '#0f1117',
    '--editor-line-highlight':'#1e2130',
    '--editor-selection':     '#3730a3',
    '--terminal-background':  '#0a0c12',
    '--sidebar-background':   '#13151f',
    '--statusbar-background': '#6366f1',
    '--ai-sidebar-background':'#141720',
  },

  typography: {
    fontFamilyUI:   '"Inter", "Segoe UI", system-ui, sans-serif',
    fontFamilyCode: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
    fontSizeBase:   '14px',
    lineHeightBase: '1.6',
  },

  spacing: {
    borderRadius:    '6px',
    sidebarWidth:    '260px',
    aiSidebarWidth:  '340px',
    panelMinHeight:  '180px',
  },
}
