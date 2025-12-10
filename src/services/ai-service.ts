import { invoke } from '@tauri-apps/api/core';

// --- DÉFINITION DES TYPES ---

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

// --- SERVICE ---

class AiService {
  /**
   * Envoie un message au Chatbot (commande existante)
   */
  async chat(message: string, context?: any): Promise<string> {
    return await invoke('ai_chat', { userMessage: message, context });
  }

  /**
   * Récupère l'état global du système IA
   */
  async getSystemStatus(): Promise<AiStatus> {
    return await invoke('ai_get_system_status');
  }

  /**
   * Teste le moteur de tokenization NLP
   */
  async testNlp(text: string): Promise<NlpResult> {
    return await invoke('ai_test_nlp', { text });
  }
}

export const aiService = new AiService();
