'use client'

import * as React from 'react'
import { useTranslations } from 'next-intl'
import { Send, Loader2, Bot, User, Sparkles } from 'lucide-react'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  streaming?: boolean
}

interface AiChatProps {
  workspaceId: string
  currentFile: string | null
  currentContent: string
}

export function AiChat({ workspaceId, currentFile, currentContent }: AiChatProps) {
  const t = useTranslations('ide')
  const [messages, setMessages] = React.useState<Message[]>([])
  const [input, setInput] = React.useState('')
  const [loading, setLoading] = React.useState(false)
  const messagesEndRef = React.useRef<HTMLDivElement>(null)
  const abortRef = React.useRef<AbortController | null>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  React.useEffect(() => {
    scrollToBottom()
  }, [messages])

  const sendMessage = async () => {
    if (!input.trim() || loading) return

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: input.trim(),
    }

    setMessages((prev) => [...prev, userMessage])
    setInput('')
    setLoading(true)

    const assistantId = crypto.randomUUID()
    const assistantMessage: Message = {
      id: assistantId,
      role: 'assistant',
      content: '',
      streaming: true,
    }
    setMessages((prev) => [...prev, assistantMessage])

    const controller = new AbortController()
    abortRef.current = controller

    try {
      const res = await fetch(`/api/v1/workspaces/${workspaceId}/ai/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        signal: controller.signal,
        body: JSON.stringify({
          message: userMessage.content,
          context: {
            file_path: currentFile,
            file_content: currentContent.slice(0, 8000),
          },
        }),
      })

      if (!res.ok || !res.body) {
        throw new Error(`HTTP ${res.status}`)
      }

      const reader = res.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() ?? ''

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6)
            if (data === '[DONE]') continue
            try {
              const parsed = JSON.parse(data)
              const delta = parsed?.delta?.text ?? parsed?.content ?? ''
              if (delta) {
                setMessages((prev) =>
                  prev.map((m) =>
                    m.id === assistantId
                      ? { ...m, content: m.content + delta }
                      : m,
                  ),
                )
              }
            } catch {
              // skip malformed SSE line
            }
          }
        }
      }
    } catch (err) {
      if ((err as Error).name !== 'AbortError') {
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantId
              ? { ...m, content: t('ai_error'), streaming: false }
              : m,
          ),
        )
      }
    } finally {
      setMessages((prev) =>
        prev.map((m) => (m.id === assistantId ? { ...m, streaming: false } : m)),
      )
      setLoading(false)
      abortRef.current = null
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }

  return (
    <div className="flex h-full flex-col border-l border-[#313244] bg-[#181825]">
      {/* Header */}
      <div className="flex h-9 shrink-0 items-center gap-2 border-b border-[#313244] px-3">
        <Sparkles className="h-3.5 w-3.5 text-[#cba6f7]" />
        <span className="text-xs font-medium text-[#cdd6f4]">{t('ai_chat')}</span>
        {currentFile && (
          <span className="ml-auto max-w-[120px] truncate text-xs text-[#45475a]">
            {currentFile.split('/').pop()}
          </span>
        )}
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full gap-3 text-center">
            <Bot className="h-8 w-8 text-[#45475a]" />
            <p className="text-sm text-[#6c7086]">{t('ai_placeholder')}</p>
          </div>
        )}

        {messages.map((msg) => (
          <div key={msg.id} className={msg.role === 'user' ? 'flex justify-end' : 'flex justify-start'}>
            <div
              className={[
                'max-w-[85%] rounded-lg px-3 py-2 text-sm',
                msg.role === 'user'
                  ? 'bg-[#89b4fa] text-[#1e1e2e]'
                  : 'bg-[#313244] text-[#cdd6f4]',
              ].join(' ')}
            >
              <div className="flex items-center gap-1.5 mb-1">
                {msg.role === 'assistant' ? (
                  <Bot className="h-3 w-3 text-[#cba6f7]" />
                ) : (
                  <User className="h-3 w-3" />
                )}
                <span className={['text-xs font-medium', msg.role === 'user' ? 'text-[#1e1e2e]/70' : 'text-[#6c7086]'].join(' ')}>
                  {msg.role === 'user' ? t('you') : 'Koda AI'}
                </span>
              </div>
              <pre className="whitespace-pre-wrap font-sans leading-relaxed">
                {msg.content}
                {msg.streaming && (
                  <span className="inline-block ml-0.5 h-3.5 w-0.5 bg-[#cba6f7] animate-pulse" />
                )}
              </pre>
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="border-t border-[#313244] p-3">
        <div className="flex gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('ai_input_placeholder')}
            rows={3}
            className="flex-1 resize-none rounded-lg border border-[#313244] bg-[#1e1e2e] px-3 py-2 text-sm text-[#cdd6f4] placeholder-[#45475a] focus:border-[#89b4fa] focus:outline-none transition-colors"
          />
          <button
            onClick={sendMessage}
            disabled={!input.trim() || loading}
            className="flex h-full items-center self-end rounded-lg bg-[#89b4fa] px-3 py-2 text-[#1e1e2e] transition-colors hover:bg-[#89b4fa]/80 disabled:cursor-not-allowed disabled:opacity-40"
          >
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Send className="h-4 w-4" />
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
