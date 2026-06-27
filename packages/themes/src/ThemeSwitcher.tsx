'use client'

import { useTheme } from './ThemeProvider'
import type { SkinId } from './types'

// Composant de sélection de thème — à intégrer dans settings ou header.
// Affiche une miniature colorée + nom pour chaque skin disponible.
export function ThemeSwitcher() {
  const { skinId, setSkin, availableSkins } = useTheme()

  return (
    <div role="radiogroup" aria-label="Choisir un thème" style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
      {availableSkins.map((s) => (
        <button
          key={s.id}
          role="radio"
          aria-checked={s.id === skinId}
          onClick={() => setSkin(s.id as SkinId)}
          title={s.description}
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: '6px',
            padding: '8px',
            border: s.id === skinId ? '2px solid var(--primary)' : '2px solid var(--border)',
            borderRadius: 'var(--radius)',
            background: 'var(--surface)',
            cursor: 'pointer',
            minWidth: '80px',
          }}
        >
          <SkinPreview skin={s} />
          <span style={{ fontSize: '11px', color: 'var(--foreground-muted)' }}>
            {s.name.replace('Koda ', '')}
          </span>
        </button>
      ))}
    </div>
  )
}

// Miniature représentant visuellement le layout du skin
function SkinPreview({ skin }: { skin: ReturnType<typeof useTheme>['availableSkins'][number] }) {
  const isTopNav = skin.layout === 'top-nav'

  return (
    <div
      aria-hidden
      style={{
        width: '64px',
        height: '44px',
        borderRadius: '4px',
        overflow: 'hidden',
        background: skin.colors['--background'],
        display: 'flex',
        flexDirection: isTopNav ? 'column' : 'row',
        position: 'relative',
      }}
    >
      {isTopNav ? (
        <>
          {/* Top nav bar */}
          <div style={{ height: '8px', background: skin.colors['--statusbar-background'], flexShrink: 0 }} />
          {/* Content area */}
          <div style={{ flex: 1, background: skin.colors['--editor-background'] }} />
        </>
      ) : (
        <>
          {/* Sidebar */}
          <div style={{ width: '18px', background: skin.colors['--sidebar-background'], flexShrink: 0 }} />
          {/* Editor */}
          <div style={{ flex: 1, background: skin.colors['--editor-background'], display: 'flex', flexDirection: 'column' }}>
            <div style={{ flex: 1 }} />
            {/* Status bar */}
            <div style={{ height: '4px', background: skin.colors['--statusbar-background'] }} />
          </div>
          {/* AI sidebar (only default/pro/light) */}
          {skin.id !== 'minimal' && (
            <div style={{ width: '16px', background: skin.colors['--ai-sidebar-background'] }} />
          )}
        </>
      )}
    </div>
  )
}
