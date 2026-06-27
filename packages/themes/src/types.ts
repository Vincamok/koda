export type LayoutVariant =
  | 'sidebar-left'   // sidebar gauche, éditeur centre, IA droite
  | 'sidebar-right'  // sidebar droite (miroir)
  | 'top-nav'        // navigation horizontale, contenu pleine largeur
  | 'minimal'        // chrome minimal, panneaux à la demande

export type DensityVariant = 'compact' | 'comfortable' | 'spacious'

export type ColorMode = 'dark' | 'light'

export interface ColorScheme {
  // Arrière-plans
  '--background': string
  '--background-secondary': string
  '--background-tertiary': string
  '--surface': string
  '--surface-hover': string
  '--surface-active': string

  // Texte
  '--foreground': string
  '--foreground-muted': string
  '--foreground-subtle': string

  // Accents
  '--primary': string
  '--primary-foreground': string
  '--primary-hover': string
  '--secondary': string
  '--secondary-foreground': string

  // Statuts
  '--success': string
  '--warning': string
  '--destructive': string
  '--info': string

  // Bordures
  '--border': string
  '--border-strong': string

  // Spécifiques IDE
  '--editor-background': string
  '--editor-line-highlight': string
  '--editor-selection': string
  '--terminal-background': string
  '--sidebar-background': string
  '--statusbar-background': string
  '--ai-sidebar-background': string
}

export interface Typography {
  fontFamilyUI: string      // Interface (labels, boutons)
  fontFamilyCode: string    // Éditeur + terminal
  fontSizeBase: string      // Base rem
  lineHeightBase: string
}

export interface Spacing {
  borderRadius: string      // rayon de bordure global
  sidebarWidth: string      // largeur sidebar fichiers
  aiSidebarWidth: string    // largeur sidebar IA
  panelMinHeight: string    // hauteur min terminal/panneau bas
}

export interface Skin {
  id: string
  name: string
  description: string
  colorMode: ColorMode
  layout: LayoutVariant
  density: DensityVariant
  colors: ColorScheme
  typography: Typography
  spacing: Spacing
  previewColor: string      // couleur hex pour la miniature dans le sélecteur
}

export type SkinId = 'default' | 'minimal' | 'pro' | 'light'
