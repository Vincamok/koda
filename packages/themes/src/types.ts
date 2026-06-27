export type LayoutVariant =
  | 'sidebar-left'   // sidebar gauche, éditeur centre, IA droite
  | 'sidebar-right'  // sidebar droite (miroir)
  | 'top-nav'        // navigation horizontale, contenu pleine largeur
  | 'minimal'        // chrome minimal, panneaux à la demande

export type DensityVariant = 'compact' | 'comfortable' | 'spacious'

export type ColorMode = 'dark' | 'light'

export interface ColorScheme {
  '--background': string
  '--background-secondary': string
  '--background-tertiary': string
  '--surface': string
  '--surface-hover': string
  '--surface-active': string
  '--foreground': string
  '--foreground-muted': string
  '--foreground-subtle': string
  '--primary': string
  '--primary-foreground': string
  '--primary-hover': string
  '--secondary': string
  '--secondary-foreground': string
  '--success': string
  '--warning': string
  '--destructive': string
  '--info': string
  '--border': string
  '--border-strong': string
  '--editor-background': string
  '--editor-line-highlight': string
  '--editor-selection': string
  '--terminal-background': string
  '--sidebar-background': string
  '--statusbar-background': string
  '--ai-sidebar-background': string
  // Extensions libres : toute propriété CSS préfixée --koda-* est acceptée
  [key: `--koda-${string}`]: string
}

export interface Typography {
  fontFamilyUI: string
  fontFamilyCode: string
  fontSizeBase: string
  lineHeightBase: string
}

export interface Spacing {
  borderRadius: string
  sidebarWidth: string
  aiSidebarWidth: string
  panelMinHeight: string
}

export interface Skin {
  id: string
  name: string
  description: string
  version: string             // SemVer — permet la compatibilité future
  author?: string
  colorMode: ColorMode
  layout: LayoutVariant
  density: DensityVariant
  colors: ColorScheme
  typography: Typography
  spacing: Spacing
  previewColor: string
  tags?: string[]             // ex: ['dark', 'high-contrast', 'community']
}

/**
 * Manifest JSON-sérialisable pour chargement dynamique de thèmes.
 * Permet l'héritage : `extends` copie le skin de base puis applique les overrides.
 */
export interface SkinManifest {
  id: string
  name: string
  description: string
  version: string
  author?: string
  tags?: string[]
  previewColor: string
  extends?: string            // id d'un skin de base enregistré dans le registre
  colorMode?: ColorMode
  layout?: LayoutVariant
  density?: DensityVariant
  colors?: Partial<ColorScheme>
  typography?: Partial<Typography>
  spacing?: Partial<Spacing>
}

export type DeepPartial<T> = { [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K] }
