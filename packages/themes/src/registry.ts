import type { Skin, SkinManifest } from './types'

type ChangeListener = (skins: Skin[]) => void

/**
 * Registre global des thèmes. Découplé du contexte React pour permettre
 * l'enregistrement depuis n'importe quel point d'entrée (CLI, tests, plugins).
 *
 * Usage :
 *   themeRegistry.register(mySkin)
 *   themeRegistry.loadManifest(manifest)         // depuis JSON externe
 *   themeRegistry.extend('default', { colors: { '--primary': '#ff0000' } })
 */
class ThemeRegistry {
  private skins = new Map<string, Skin>()
  private listeners = new Set<ChangeListener>()

  register(skin: Skin): void {
    this.skins.set(skin.id, skin)
    this.notify()
  }

  unregister(id: string): void {
    this.skins.delete(id)
    this.notify()
  }

  get(id: string): Skin | undefined {
    return this.skins.get(id)
  }

  getOrDefault(id: string, fallbackId = 'default'): Skin {
    return this.skins.get(id) ?? this.skins.get(fallbackId) ?? [...this.skins.values()][0]
  }

  list(): Skin[] {
    return [...this.skins.values()]
  }

  has(id: string): boolean {
    return this.skins.has(id)
  }

  /**
   * Crée un nouveau skin en héritant d'un skin de base et en appliquant des overrides.
   * Le skin résultant est enregistré automatiquement si un id est fourni dans les overrides.
   */
  extend(baseId: string, overrides: Partial<Omit<Skin, 'colors' | 'typography' | 'spacing'>> & {
    colors?: Partial<Skin['colors']>
    typography?: Partial<Skin['typography']>
    spacing?: Partial<Skin['spacing']>
  }): Skin {
    const base = this.skins.get(baseId)
    if (!base) throw new Error(`Skin de base "${baseId}" introuvable dans le registre`)

    const result: Skin = {
      ...base,
      ...overrides,
      colors:     { ...base.colors,     ...overrides.colors },
      typography: { ...base.typography, ...overrides.typography },
      spacing:    { ...base.spacing,    ...overrides.spacing },
    }

    if (overrides.id && overrides.id !== baseId) {
      this.register(result)
    }

    return result
  }

  /**
   * Charge un skin depuis un manifest JSON (format sérialisable, stockable en DB).
   * Si `extends` est défini, le skin de base est utilisé comme point de départ.
   */
  loadManifest(manifest: SkinManifest): Skin {
    const base = manifest.extends ? this.skins.get(manifest.extends) : undefined

    const skin: Skin = {
      id:          manifest.id,
      name:        manifest.name,
      description: manifest.description,
      version:     manifest.version,
      author:      manifest.author,
      tags:        manifest.tags,
      previewColor: manifest.previewColor,
      colorMode:   manifest.colorMode   ?? base?.colorMode   ?? 'dark',
      layout:      manifest.layout      ?? base?.layout      ?? 'sidebar-left',
      density:     manifest.density     ?? base?.density     ?? 'comfortable',
      colors:      { ...(base?.colors     ?? {}), ...manifest.colors }     as Skin['colors'],
      typography:  { ...(base?.typography ?? {}), ...manifest.typography } as Skin['typography'],
      spacing:     { ...(base?.spacing    ?? {}), ...manifest.spacing }    as Skin['spacing'],
    }

    this.register(skin)
    return skin
  }

  /**
   * Charge plusieurs manifests depuis une URL (marketplace communautaire, futur).
   */
  async loadFromUrl(url: string): Promise<Skin[]> {
    const res = await fetch(url)
    if (!res.ok) throw new Error(`Échec chargement thèmes depuis ${url}: ${res.status}`)
    const manifests: SkinManifest[] = await res.json()
    return manifests.map((m) => this.loadManifest(m))
  }

  onChange(listener: ChangeListener): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  private notify(): void {
    const skins = this.list()
    this.listeners.forEach((l) => l(skins))
  }
}

export const themeRegistry = new ThemeRegistry()
