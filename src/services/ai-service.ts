import { invoke } from '@tauri-apps/api/core';
import type { CreatedArtifact } from '@/types/ai.types';

// Structure retournée par le Backend Rust (AgentResult)
export interface AgentResult {
  type: 'text' | 'action' | 'file';
  content: string;
  artifacts?: CreatedArtifact[];
  metadata?: Record<string, unknown>;
}

export interface AiStatus {
  llm_connected: boolean;
  llm_model: string;
  context_documents: number;
  active_agents: string[];
}

export interface NlpResult {
  token_count: number;
  tokens: string[];
}

class AiService {
  /**
   * Envoie un message à l'Orchestrateur IA.
   */
  async chat(userInput: string): Promise<AgentResult> {
    try {
      return await invoke<AgentResult>('ai_chat', {
        userInput,
      });
    } catch (error) {
      console.error('[AiService] Chat error:', error);
      throw error;
    }
  }

  /**
   * Réinitialise la mémoire conversationnelle côté Backend.
   */
  async resetMemory(): Promise<void> {
    try {
      await invoke('ai_reset');
      console.log('[AiService] Mémoire réinitialisée.');
    } catch (error) {
      console.error('[AiService] Reset error:', error);
      throw error;
    }
  }

  /**
   * Récupère le statut (ou un mock si la commande n'est pas encore implémentée).
   */
  async getSystemStatus(): Promise<AiStatus> {
    try {
      return await invoke<AiStatus>('ai_get_system_status');
    } catch (error) {
      // CORRECTION 2 : On utilise la variable 'error' dans le log pour éviter "unused vars"
      console.warn('[AiService] Status command not found (using mock). details:', error);

      return {
        llm_connected: true,
        llm_model: 'Llama-3-Local',
        context_documents: 12,
        active_agents: ['Orchestrator'],
      };
    }
  }

  async testNlp(text: string): Promise<NlpResult> {
    try {
      return await invoke<NlpResult>('ai_test_nlp', { text });
    } catch {
      return { token_count: 0, tokens: [] };
    }
  }
}

export const aiService = new AiService();
