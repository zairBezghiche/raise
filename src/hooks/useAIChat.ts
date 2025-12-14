import { useCallback } from 'react';
import { useAiStore, ChatMessage } from '@/store/ai-store';
import { useSettingsStore } from '@/store/settings-store';
import { invoke } from '@tauri-apps/api/core';

function genId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

// On garde l'interface exportée au cas où on en aurait besoin ailleurs ou plus tard
export interface UseAIChatOptions {
  systemPrompt?: string;
}

// CORRECTION : Suppression du paramètre inutilisé '_options'
export function useAIChat() {
  const { messages, isThinking, error, addMessage, clear, setThinking, setError } = useAiStore();
  const { aiBackend } = useSettingsStore();

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim();
      if (!trimmed) return;

      const userMsg: ChatMessage = {
        id: genId(),
        role: 'user',
        content: trimmed,
        createdAt: new Date().toISOString(),
      };
      addMessage(userMsg);

      setThinking(true);
      setError(undefined);

      try {
        let replyText: string;

        if (aiBackend === 'mock') {
          replyText = `[mock] Réponse : "${trimmed}"`;
        } else {
          console.log('Envoi vers Rust (ai_chat)...');
          replyText = await invoke<string>('ai_chat', {
            userInput: trimmed,
          });
        }

        const assistantMsg: ChatMessage = {
          id: genId(),
          role: 'assistant',
          content: replyText,
          createdAt: new Date().toISOString(),
        };
        addMessage(assistantMsg);
      } catch (e: unknown) {
        console.error('Erreur AI:', e);
        const errorMessage = e instanceof Error ? e.message : String(e);
        setError(errorMessage);
      } finally {
        setThinking(false);
      }
    },
    [addMessage, setThinking, setError, aiBackend],
  );

  return {
    messages,
    isThinking,
    error,
    sendMessage,
    clear,
  };
}
