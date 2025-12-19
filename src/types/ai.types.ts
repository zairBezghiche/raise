// FICHIER : src/types/ai.types.ts

// --- ARTEFACTS (Correspondance Backend Rust) ---
// C'est ici qu'on définit la structure des "Cartes"
export interface CreatedArtifact {
  id: string;
  name: string;
  layer: string; // Ex: "SA", "LA", "OA", "TRANSVERSE"...
  element_type: string; // Ex: "Function", "Class", "Requirement"...
  path: string; // Chemin relatif (ex: "un2/sa/functions/123.json")
}

export interface AgentResult {
  message: string;
  artifacts: CreatedArtifact[];
}

// --- CHAT ---

export type AiRole = 'user' | 'assistant' | 'system';

export interface ChatMessage {
  id: string;
  role: AiRole;
  content: string;
  createdAt: string;
  artifacts?: CreatedArtifact[]; // <--- NOUVEAU : Pour afficher les cartes
  meta?: Record<string, unknown>;
}

// --- SYSTÈME & STATUS ---

export interface AiStatus {
  llm_connected: boolean;
  llm_model: string;
  context_documents: number;
  active_agents: string[];
}

// --- NLP ---

export interface NlpResult {
  token_count: number;
  tokens: string[];
  entities?: Array<{ text: string; label: string }>;
}

// --- CONFIGURATION ---

export type AiBackendType = 'mock' | 'tauri-local' | 'remote-api';
