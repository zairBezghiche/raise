// Définition d'un élément du modèle (Logical Component, Function, etc.)
export interface ModelElement {
  name: string;
  kind: string;
  properties: Record<string, string>;
}

// Le Modèle Cognitif complet (ce qu'on envoie au WASM)
export interface CognitiveModel {
  id: string;
  elements: Record<string, ModelElement>;
  metadata: Record<string, string>;
}

// Le Rapport d'Analyse (ce qu'on reçoit du WASM)
export interface AnalysisReport {
  block_id: string;
  status: 'Success' | 'Warning' | 'Failure';
  messages: string[];
  timestamp: number;
}
