import { useCallback } from 'react'
import { useAiStore, ChatMessage } from '@/store/ai-store'
import { useSettingsStore } from '@/store/settings-store'
// import { invoke } from '@tauri-apps/api/core' // à activer quand tu auras la commande Tauri IA

function genId(): string {
  return `${Date.now().toString(36)}-${Math.random()
    .toString(36)
    .slice(2)}`
}

export interface UseAIChatOptions {
  systemPrompt?: string
}

export function useAIChat(options?: UseAIChatOptions) {
  const { messages, isThinking, error, addMessage, clear, setThinking, setError } =
    useAiStore()
  const { aiBackend } = useSettingsStore()

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim()
      if (!trimmed) return

      const userMsg: ChatMessage = {
        id: genId(),
        role: 'user',
        content: trimmed,
        createdAt: new Date().toISOString(),
      }
      addMessage(userMsg)
      setThinking(true)
      setError(undefined)

      try {
        let replyText: string

        if (aiBackend === 'mock') {
          replyText = `[mock] Réponse de l’assistant GenAptitude pour : "${trimmed}"`
        } else {
          // TODO: brancher sur une commande Tauri (ex: "ai_chat")
          // const res = await invoke<string>('ai_chat', {
          //   input: trimmed,
          //   systemPrompt: options?.systemPrompt,
          //   history: messages,
          // })
          // replyText = res
          replyText = `[TODO:${aiBackend}] backend IA non encore implémenté`
        }

        const assistantMsg: ChatMessage = {
          id: genId(),
          role: 'assistant',
          content: replyText,
          createdAt: new Date().toISOString(),
        }
        addMessage(assistantMsg)
      } catch (e: any) {
        setError(e?.message ?? 'Erreur IA')
      } finally {
        setThinking(false)
      }
    },
    [addMessage, setThinking, setError, aiBackend, messages, options?.systemPrompt],
  )

  return {
    messages,
    isThinking,
    error,
    sendMessage,
    clear,
  }
}
