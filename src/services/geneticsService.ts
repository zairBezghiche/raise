import { invoke } from '@tauri-apps/api/core';

export interface GeneticsParams {
  population_size: number;
  generations: number;
  mutation_rate: number;
}

export interface OptimizationResult {
  best_score: number;
  duration_ms: number;
  improvement_log: number[];
  best_candidate_id: string;
}

class GeneticsService {
  async runOptimization(params: GeneticsParams, model: any): Promise<OptimizationResult> {
    try {
      // Les clés de params doivent correspondre au snake_case du Rust si on n'utilise pas rename_all
      // Ici, j'ai utilisé les mêmes noms (snake_case) dans l'interface TS pour simplifier.
      return await invoke<OptimizationResult>('run_genetic_optimization', {
        params: params,
        model: model,
      });
    } catch (error) {
      console.error('❌ Erreur génétique:', error);
      throw error;
    }
  }
}

export const geneticsService = new GeneticsService();
