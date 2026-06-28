'use client'

import * as React from 'react'

const BREAKPOINTS = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536,
} as const

type Breakpoint = keyof typeof BREAKPOINTS

/**
 * Returns true when the viewport is at least as wide as the given breakpoint.
 * Safe on SSR: returns false until hydrated.
 */
export function useBreakpoint(bp: Breakpoint): boolean {
  const [matches, setMatches] = React.useState(false)

  React.useEffect(() => {
    const mq = window.matchMedia(`(min-width: ${BREAKPOINTS[bp]}px)`)
    setMatches(mq.matches)
    const handler = (e: MediaQueryListEvent) => setMatches(e.matches)
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [bp])

  return matches
}

/** Returns true when viewport is narrower than the `md` breakpoint (< 768px). */
export function useIsMobile(): boolean {
  const isMd = useBreakpoint('md')
  return !isMd
}
