import { invoke } from '@tauri-apps/api/core'; // Tauri v2
import { CognitiveModel, AnalysisReport } from '../types/cognitive';

class CognitiveService {
  /**
   * Envoie un modèle au moteur de plugins pour analyse via WASM.
   */
  async runConsistencyCheck(model: CognitiveModel): Promise<AnalysisReport> {
    try {
      console.log('📤 Envoi du modèle au bloc cognitif...', model);

      // CORRECTION ICI : Tauri attend du camelCase (modelJson) pour mapper vers le snake_case Rust (model_json)
      const jsonString = await invoke<string>('run_consistency_analysis', {
        modelJson: model,
      });

      const report: AnalysisReport = JSON.parse(jsonString);
      return report;
    } catch (error) {
      console.error('❌ Erreur service cognitif:', error);
      throw error;
    }
  }
}

export const cognitiveService = new CognitiveService();
