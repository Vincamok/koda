import type { Skin } from '../types'

// Thème clair pour les environnements lumineux ou les préférences utilisateur.
// Même layout que Default mais avec une palette inversée et des contrastes doux.
export const lightSkin: Skin = {
  id: 'light',
  name: 'Koda Light',
  description: 'Interface claire et aérée. Idéale pour les environnements très lumineux.',
  colorMode: 'light',
  layout: 'sidebar-left',
  density: 'comfortable',
  previewColor: '#f8fafc',

  colors: {
    '--background':           '#f8fafc',
    '--background-secondary': '#f1f5f9',
    '--background-tertiary':  '#e2e8f0',
    '--surface':              '#ffffff',
    '--surface-hover':        '#f1f5f9',
    '--surface-active':       '#e2e8f0',

    '--foreground':           '#0f172a',
    '--foreground-muted':     '#475569',
    '--foreground-subtle':    '#94a3b8',

    '--primary':              '#4f46e5',
    '--primary-foreground':   '#ffffff',
    '--primary-hover':        '#4338ca',
    '--secondary':            '#e2e8f0',
    '--secondary-foreground': '#1e293b',

    '--success':              '#16a34a',
    '--warning':              '#d97706',
    '--destructive':          '#dc2626',
    '--info':                 '#2563eb',

    '--border':               '#e2e8f0',
    '--border-strong':        '#cbd5e1',

    '--editor-background':    '#ffffff',
    '--editor-line-highlight':'#f8fafc',
    '--editor-selection':     '#c7d2fe',
    '--terminal-background':  '#1e293b',
    '--sidebar-background':   '#f1f5f9',
    '--statusbar-background': '#4f46e5',
    '--ai-sidebar-background':'#f8fafc',
  },

  typography: {
    fontFamilyUI:   '"Inter", "Segoe UI", system-ui, sans-serif',
    fontFamilyCode: '"JetBrains Mono", "Fira Code", monospace',
    fontSizeBase:   '14px',
    lineHeightBase: '1.6',
  },

  spacing: {
    borderRadius:    '8px',
    sidebarWidth:    '260px',
    aiSidebarWidth:  '340px',
    panelMinHeight:  '180px',
  },
}
