import type { Skin } from '../types'

// Navigation horizontale, pas de sidebar permanente, panneaux à la demande.
// Adapté aux petits écrans et aux sessions de travail centrées sur l'éditeur.
export const minimalSkin: Skin = {
  id: 'minimal',
  name: 'Koda Minimal',
  description: 'Navigation en haut, éditeur pleine largeur. Panneaux ouverts à la demande.',
  colorMode: 'dark',
  layout: 'top-nav',
  density: 'spacious',
  previewColor: '#18181b',

  colors: {
    '--background':           '#18181b',
    '--background-secondary': '#27272a',
    '--background-tertiary':  '#3f3f46',
    '--surface':              '#27272a',
    '--surface-hover':        '#3f3f46',
    '--surface-active':       '#52525b',

    '--foreground':           '#fafafa',
    '--foreground-muted':     '#a1a1aa',
    '--foreground-subtle':    '#71717a',

    '--primary':              '#a78bfa',
    '--primary-foreground':   '#18181b',
    '--primary-hover':        '#8b5cf6',
    '--secondary':            '#3f3f46',
    '--secondary-foreground': '#d4d4d8',

    '--success':              '#4ade80',
    '--warning':              '#fbbf24',
    '--destructive':          '#f87171',
    '--info':                 '#60a5fa',

    '--border':               '#3f3f46',
    '--border-strong':        '#52525b',

    '--editor-background':    '#18181b',
    '--editor-line-highlight':'#27272a',
    '--editor-selection':     '#4c1d95',
    '--terminal-background':  '#09090b',
    '--sidebar-background':   '#27272a',
    '--statusbar-background': '#27272a',
    '--ai-sidebar-background':'#27272a',
  },

  typography: {
    fontFamilyUI:   '"Inter", system-ui, sans-serif',
    fontFamilyCode: '"JetBrains Mono", monospace',
    fontSizeBase:   '14px',
    lineHeightBase: '1.7',
  },

  spacing: {
    borderRadius:    '8px',
    sidebarWidth:    '240px',
    aiSidebarWidth:  '320px',
    panelMinHeight:  '200px',
  },
}
