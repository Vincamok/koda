export type { Skin, SkinId, LayoutVariant, DensityVariant, ColorMode, ColorScheme, Typography, Spacing } from './types'
export { defaultSkin } from './themes/default'
export { minimalSkin } from './themes/minimal'
export { proSkin } from './themes/pro'
export { lightSkin } from './themes/light'

import { defaultSkin } from './themes/default'
import { minimalSkin } from './themes/minimal'
import { proSkin } from './themes/pro'
import { lightSkin } from './themes/light'
import type { Skin, SkinId } from './types'

export const SKINS: Record<SkinId, Skin> = {
  default: defaultSkin,
  minimal: minimalSkin,
  pro:     proSkin,
  light:   lightSkin,
}

export const DEFAULT_SKIN_ID: SkinId = 'default'

export function getSkin(id: string): Skin {
  return SKINS[id as SkinId] ?? defaultSkin
}

export function applySkin(skin: Skin, root: HTMLElement = document.documentElement): void {
  // Applique toutes les CSS custom properties
  for (const [prop, value] of Object.entries(skin.colors)) {
    root.style.setProperty(prop, value)
  }
  // Applique les classes de layout et densité
  root.dataset.layout  = skin.layout
  root.dataset.density = skin.density
  root.dataset.colorMode = skin.colorMode

  // Variables de typographie et espacement
  root.style.setProperty('--font-ui',        skin.typography.fontFamilyUI)
  root.style.setProperty('--font-code',      skin.typography.fontFamilyCode)
  root.style.setProperty('--font-size-base', skin.typography.fontSizeBase)
  root.style.setProperty('--line-height',    skin.typography.lineHeightBase)
  root.style.setProperty('--radius',         skin.spacing.borderRadius)
  root.style.setProperty('--sidebar-width',  skin.spacing.sidebarWidth)
  root.style.setProperty('--ai-sidebar-width', skin.spacing.aiSidebarWidth)
  root.style.setProperty('--panel-min-height', skin.spacing.panelMinHeight)
}
