import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAiStore } from '@/store/ai-store';
import { ChatMessage, AgentResult } from '@/types/ai.types';

export function useAIChat() {
  // On utilise le store global pour l'état (messages, loading...)
  const { messages, isThinking, error, addMessage, setThinking, setError, clear } = useAiStore();

  const sendMessage = useCallback(
    async (text: string) => {
      // Nettoyage de l'input
      if (!text.trim()) return;

      // 1. On affiche tout de suite le message de l'utilisateur
      const userMsg: ChatMessage = {
        id: Date.now().toString(),
        role: 'user',
        content: text,
        createdAt: new Date().toISOString(),
      };
      addMessage(userMsg);

      setThinking(true);
      setError(undefined);

      try {
        console.log('🚀 Envoi vers Rust (ai_chat)...');

        // 2. Appel Backend
        const response = await invoke<AgentResult>('ai_chat', { userInput: text });

        // 3. On crée la réponse de l'assistant avec les cartes (artefacts)
        const aiMsg: ChatMessage = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: response.message, // Le texte explicatif
          artifacts: response.artifacts, // Les cartes visuelles
          createdAt: new Date().toISOString(),
        };
        addMessage(aiMsg);
      } catch (err) {
        console.error('❌ Erreur AI:', err);
        setError(String(err));

        const errorMsg: ChatMessage = {
          id: Date.now().toString(),
          role: 'assistant',
          content: `⚠️ Erreur système : ${err}`,
          createdAt: new Date().toISOString(),
        };
        addMessage(errorMsg);
      } finally {
        setThinking(false);
      }
    },
    [addMessage, setThinking, setError],
  );

  return { messages, isThinking, error, sendMessage, clear };
}
