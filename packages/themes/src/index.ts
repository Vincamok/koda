export type { Skin, SkinManifest, SkinId, LayoutVariant, DensityVariant, ColorMode, ColorScheme, Typography, Spacing, DeepPartial } from './types'
export { themeRegistry } from './registry'
export { defaultSkin } from './themes/default'
export { minimalSkin } from './themes/minimal'
export { proSkin } from './themes/pro'
export { lightSkin } from './themes/light'

import { themeRegistry } from './registry'
import { defaultSkin } from './themes/default'
import { minimalSkin } from './themes/minimal'
import { proSkin } from './themes/pro'
import { lightSkin } from './themes/light'
import type { Skin } from './types'

// Enregistrement des skins built-in au chargement du module
themeRegistry.register(defaultSkin)
themeRegistry.register(minimalSkin)
themeRegistry.register(proSkin)
themeRegistry.register(lightSkin)

export const DEFAULT_SKIN_ID = 'default'

export function applySkin(skin: Skin, root: HTMLElement = document.documentElement): void {
  for (const [prop, value] of Object.entries(skin.colors)) {
    root.style.setProperty(prop, value as string)
  }
  root.dataset.layout    = skin.layout
  root.dataset.density   = skin.density
  root.dataset.colorMode = skin.colorMode
  root.style.setProperty('--font-ui',          skin.typography.fontFamilyUI)
  root.style.setProperty('--font-code',        skin.typography.fontFamilyCode)
  root.style.setProperty('--font-size-base',   skin.typography.fontSizeBase)
  root.style.setProperty('--line-height',      skin.typography.lineHeightBase)
  root.style.setProperty('--radius',           skin.spacing.borderRadius)
  root.style.setProperty('--sidebar-width',    skin.spacing.sidebarWidth)
  root.style.setProperty('--ai-sidebar-width', skin.spacing.aiSidebarWidth)
  root.style.setProperty('--panel-min-height', skin.spacing.panelMinHeight)
}

// Re-export type alias pour rétrocompatibilité
export type SkinId = string
