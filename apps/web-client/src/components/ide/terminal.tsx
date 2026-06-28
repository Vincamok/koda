'use client'

import * as React from 'react'

interface TerminalProps {
  workspaceId: string
}

// xterm is ESM-only and can't SSR — dynamic import in useEffect
export function Terminal({ workspaceId }: TerminalProps) {
  const containerRef = React.useRef<HTMLDivElement>(null)
  const termRef = React.useRef<import('xterm').Terminal | null>(null)

  React.useEffect(() => {
    if (!containerRef.current) return

    let term: import('xterm').Terminal
    let fitAddon: import('xterm-addon-fit').FitAddon

    const init = async () => {
      const { Terminal: XTerm } = await import('xterm')
      const { FitAddon } = await import('xterm-addon-fit')
      const { WebLinksAddon } = await import('xterm-addon-web-links')

      term = new XTerm({
        fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace',
        fontSize: 13,
        theme: {
          background: '#11111b',
          foreground: '#cdd6f4',
          cursor: '#f5c2e7',
          black: '#45475a',
          red: '#f38ba8',
          green: '#a6e3a1',
          yellow: '#f9e2af',
          blue: '#89b4fa',
          magenta: '#cba6f7',
          cyan: '#94e2d5',
          white: '#bac2de',
          brightBlack: '#585b70',
          brightRed: '#f38ba8',
          brightGreen: '#a6e3a1',
          brightYellow: '#f9e2af',
          brightBlue: '#89b4fa',
          brightMagenta: '#cba6f7',
          brightCyan: '#94e2d5',
          brightWhite: '#a6adc8',
        },
        cursorBlink: true,
        scrollback: 5000,
        allowTransparency: false,
      })

      fitAddon = new FitAddon()
      term.loadAddon(fitAddon)
      term.loadAddon(new WebLinksAddon())

      if (containerRef.current) {
        term.open(containerRef.current)
        fitAddon.fit()
        termRef.current = term
      }

      // Connect to workspace terminal via WebSocket
      const wsProtocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
      const ws = new WebSocket(
        `${wsProtocol}//${location.host}/api/v1/ws/${workspaceId}/terminal`,
      )

      ws.onopen = () => {
        term.writeln('\x1b[32m✓ Connected to workspace terminal\x1b[0m')
        term.writeln('')
      }

      ws.onmessage = (e) => {
        term.write(e.data)
      }

      ws.onclose = () => {
        term.writeln('\r\n\x1b[33m⚡ Terminal disconnected\x1b[0m')
      }

      ws.onerror = () => {
        term.writeln('\r\n\x1b[31m✗ Terminal connection failed\x1b[0m')
      }

      term.onData((data) => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(data)
        }
      })

      // Send resize event to backend (0x01 + u16 cols BE + u16 rows BE)
      const sendResize = () => {
        fitAddon.fit()
        if (ws.readyState === WebSocket.OPEN) {
          const { cols, rows } = term
          const buf = new Uint8Array(5)
          buf[0] = 0x01
          new DataView(buf.buffer).setUint16(1, cols, false)
          new DataView(buf.buffer).setUint16(3, rows, false)
          ws.send(buf)
        }
      }

      const resizeObs = new ResizeObserver(() => sendResize())
      if (containerRef.current) {
        resizeObs.observe(containerRef.current)
      }

      return () => {
        ws.close()
        resizeObs.disconnect()
        term.dispose()
      }
    }

    const cleanup = init()
    return () => {
      cleanup.then((fn) => fn?.())
    }
  }, [workspaceId])

  return (
    <div className="flex h-full flex-col bg-[#11111b]">
      <div className="flex h-7 shrink-0 items-center border-b border-[#313244] px-3">
        <span className="text-xs text-[#6c7086]">Terminal</span>
      </div>
      <div ref={containerRef} className="flex-1 p-1" />
    </div>
  )
}
