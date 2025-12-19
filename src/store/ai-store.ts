import { create } from 'zustand';
import { ChatMessage } from '@/types/ai.types'; // <--- Import centralisé

export interface AiStoreState {
  messages: ChatMessage[];
  isThinking: boolean;
  error?: string;

  addMessage: (msg: ChatMessage) => void;
  clear: () => void;
  setThinking: (value: boolean) => void;
  setError: (error?: string) => void;
}

export const useAiStore = create<AiStoreState>((set) => ({
  messages: [],
  isThinking: false,
  error: undefined,

  addMessage: (msg) =>
    set((state) => ({
      messages: [...state.messages, msg],
    })),

  clear: () => set({ messages: [], error: undefined }),

  setThinking: (value) => set({ isThinking: value }),

  setError: (error) => set({ error }),
}));
