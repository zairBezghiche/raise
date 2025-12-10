import { invoke } from '@tauri-apps/api/core';

export interface GeneratedFile {
  filename: string;
  content: string;
  language: 'rust' | 'python' | 'cpp';
}

class CodegenService {
  /**
   * Demande au backend de générer le code source pour le modèle donné
   */
  async generateCode(language: string, model: any): Promise<string> {
    try {
      // Appel à la commande Rust définie à l'étape 1
      const result = await invoke<string>('generate_source_code', {
        language: language,
        model: model,
      });
      return result;
    } catch (error) {
      console.error('❌ Erreur génération de code:', error);
      throw error;
    }
  }
}

export const codegenService = new CodegenService();
