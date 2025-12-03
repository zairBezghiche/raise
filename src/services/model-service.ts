// FICHIER : src/services/model-service.ts

import { invoke } from '@tauri-apps/api/core';
import type { ProjectModel } from '@/types/model.types';

export class ModelService {
  /**
   * Charge le modèle complet depuis le backend Rust via le thread dédié.
   */
  async loadProjectModel(space: string, db: string): Promise<ProjectModel> {
    try {
      console.log(`[ModelService] Loading project ${space}/${db}...`);
      const start = performance.now();

      // Appel à la commande Rust 'load_project_model'
      const model = await invoke<ProjectModel>('load_project_model', {
        space,
        db,
      });

      const duration = (performance.now() - start).toFixed(0);
      console.log(`[ModelService] Loaded ${model.meta.elementCount} elements in ${duration}ms`);

      return model;
    } catch (error) {
      console.error('[ModelService] Failed to load project:', error);
      throw error;
    }
  }
}

export const modelService = new ModelService();
