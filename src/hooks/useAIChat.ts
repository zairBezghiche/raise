import { useCallback } from 'react';
import { useAiStore, ChatMessage } from '@/store/ai-store';
import { useSettingsStore } from '@/store/settings-store';
// Import nécessaire pour communiquer avec le backend Rust
import { invoke } from '@tauri-apps/api/core';

function genId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

export interface UseAIChatOptions {
  systemPrompt?: string;
}

export function useAIChat(options?: UseAIChatOptions) {
  const { messages, isThinking, error, addMessage, clear, setThinking, setError } = useAiStore();
  // On récupère le réglage (par défaut "mock", il faudra peut-être le forcer à "tauri-local" ou le changer dans l'UI)
  const { aiBackend } = useSettingsStore();

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim();
      if (!trimmed) return;

      // 1. Ajouter le message de l'utilisateur à l'UI tout de suite
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

        // 2. Choix du backend
        // Pour tester immédiatement, vous pouvez changer la condition ci-dessous
        // ou modifier 'aiBackend' dans le settings-store.
        if (aiBackend === 'mock') {
          // Pour le test, on force l'appel réel si vous n'avez pas encore l'UI de settings
          // Décommentez la ligne suivante pour contourner le store le temps du test :
          // replyText = await invoke('ai_chat', { userInput: trimmed });

          // Sinon, comportement par défaut :
          replyText = `[mock] Réponse : "${trimmed}"`;
        } else {
          // --- APPEL RÉEL AU BACKEND RUST ---
          console.log('Envoi vers Rust (ai_chat)...');

          // La commande Rust s'appelle 'ai_chat'
          // L'argument Rust est 'user_input', ici on passe 'userInput' (convention Tauri)
          replyText = await invoke<string>('ai_chat', {
            userInput: trimmed,
          });
        }

        // 3. Ajouter la réponse de l'assistant
        const assistantMsg: ChatMessage = {
          id: genId(),
          role: 'assistant',
          content: replyText,
          createdAt: new Date().toISOString(),
        };
        addMessage(assistantMsg);
      } catch (e: any) {
        console.error('Erreur AI:', e);
        setError(typeof e === 'string' ? e : e?.message ?? 'Erreur inconnue');
      } finally {
        setThinking(false);
      }
    },
    [addMessage, setThinking, setError, aiBackend, messages, options?.systemPrompt],
  );

  return {
    messages,
    isThinking,
    error,
    sendMessage,
    clear,
  };
}
