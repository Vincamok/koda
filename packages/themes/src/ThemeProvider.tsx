'use client'

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { themeRegistry, applySkin, DEFAULT_SKIN_ID } from './index'
import type { Skin } from './types'

interface ThemeContextValue {
  skin: Skin
  skinId: string
  setSkin: (id: string) => void
  availableSkins: Skin[]
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

const STORAGE_KEY = 'koda-skin'

interface ThemeProviderProps {
  children: ReactNode
  defaultSkinId?: string
  onSkinChange?: (id: string) => Promise<void>
}

export function ThemeProvider({ children, defaultSkinId, onSkinChange }: ThemeProviderProps) {
  const [skinId, setSkinIdState] = useState<string>(() => {
    if (typeof window === 'undefined') return defaultSkinId ?? DEFAULT_SKIN_ID
    return localStorage.getItem(STORAGE_KEY) ?? defaultSkinId ?? DEFAULT_SKIN_ID
  })

  const [availableSkins, setAvailableSkins] = useState<Skin[]>(() => themeRegistry.list())

  // Écoute les enregistrements dynamiques de nouveaux thèmes
  useEffect(() => {
    return themeRegistry.onChange(setAvailableSkins)
  }, [])

  const skin = themeRegistry.getOrDefault(skinId)

  useEffect(() => {
    applySkin(skin)
  }, [skin])

  function setSkin(id: string) {
    setSkinIdState(id)
    localStorage.setItem(STORAGE_KEY, id)
    applySkin(themeRegistry.getOrDefault(id))
    onSkinChange?.(id)
  }

  return (
    <ThemeContext.Provider value={{ skin, skinId, setSkin, availableSkins }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme doit être utilisé dans un ThemeProvider')
  return ctx
}
