'use client'

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { SKINS, DEFAULT_SKIN_ID, getSkin, applySkin } from './index'
import type { Skin, SkinId } from './types'

interface ThemeContextValue {
  skin: Skin
  skinId: SkinId
  setSkin: (id: SkinId) => void
  availableSkins: Skin[]
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

const STORAGE_KEY = 'koda-skin'

interface ThemeProviderProps {
  children: ReactNode
  defaultSkinId?: SkinId
  // Si fourni, persiste le choix en DB (appel API au changement)
  onSkinChange?: (id: SkinId) => Promise<void>
}

export function ThemeProvider({ children, defaultSkinId, onSkinChange }: ThemeProviderProps) {
  const [skinId, setSkinIdState] = useState<SkinId>(() => {
    if (typeof window === 'undefined') return defaultSkinId ?? DEFAULT_SKIN_ID
    const stored = localStorage.getItem(STORAGE_KEY) as SkinId | null
    return stored ?? defaultSkinId ?? DEFAULT_SKIN_ID
  })

  const skin = getSkin(skinId)

  useEffect(() => {
    applySkin(skin)
  }, [skin])

  function setSkin(id: SkinId) {
    setSkinIdState(id)
    localStorage.setItem(STORAGE_KEY, id)
    applySkin(getSkin(id))
    onSkinChange?.(id)
  }

  return (
    <ThemeContext.Provider value={{ skin, skinId, setSkin, availableSkins: Object.values(SKINS) }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme doit être utilisé dans un ThemeProvider')
  return ctx
}
