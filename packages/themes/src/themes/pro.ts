import type { Skin } from '../types'

// Layout dense inspiré de VS Code : activity bar gauche, sidebar, éditeur,
// panneau bas, statusbar. Maximum d'information à l'écran.
export const proSkin: Skin = {
  id: 'pro',
  name: 'Koda Pro',
  description: 'Layout dense VS Code-like. Activity bar + sidebar + éditeur + terminal intégré.',
  colorMode: 'dark',
  layout: 'sidebar-left',
  density: 'compact',
  previewColor: '#1e1e2e',

  colors: {
    '--background':           '#1e1e2e',
    '--background-secondary': '#181825',
    '--background-tertiary':  '#313244',
    '--surface':              '#313244',
    '--surface-hover':        '#45475a',
    '--surface-active':       '#585b70',

    '--foreground':           '#cdd6f4',
    '--foreground-muted':     '#a6adc8',
    '--foreground-subtle':    '#6c7086',

    '--primary':              '#cba6f7',
    '--primary-foreground':   '#1e1e2e',
    '--primary-hover':        '#b4befe',
    '--secondary':            '#313244',
    '--secondary-foreground': '#cdd6f4',

    '--success':              '#a6e3a1',
    '--warning':              '#f9e2af',
    '--destructive':          '#f38ba8',
    '--info':                 '#89b4fa',

    '--border':               '#313244',
    '--border-strong':        '#45475a',

    '--editor-background':    '#1e1e2e',
    '--editor-line-highlight':'#2a2a3c',
    '--editor-selection':     '#585b70',
    '--terminal-background':  '#181825',
    '--sidebar-background':   '#181825',
    '--statusbar-background': '#181825',
    '--ai-sidebar-background':'#1e1e2e',
  },

  typography: {
    fontFamilyUI:   '"Inter", system-ui, sans-serif',
    fontFamilyCode: '"JetBrains Mono", "Cascadia Code", monospace',
    fontSizeBase:   '13px',
    lineHeightBase: '1.5',
  },

  spacing: {
    borderRadius:    '4px',
    sidebarWidth:    '240px',
    aiSidebarWidth:  '360px',
    panelMinHeight:  '160px',
  },
}
