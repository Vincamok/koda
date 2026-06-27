export const SUPPORTED_LOCALES = ['fr', 'en', 'es', 'de'] as const
export type Locale = typeof SUPPORTED_LOCALES[number]
export const DEFAULT_LOCALE: Locale = 'fr'

export function isValidLocale(l: string): l is Locale {
  return SUPPORTED_LOCALES.includes(l as Locale)
}
