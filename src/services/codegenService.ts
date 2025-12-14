import { invoke } from '@tauri-apps/api/core';

export interface GeneratedFile {
  filename: string;
  content: string;
  language: 'rust' | 'python' | 'cpp';
}

class CodegenService {
  /**
   * Demande au backend de générer le code source pour le modèle donné.
   * @param language Langage cible
   * @param model Le modèle JSON complet
   */
  // Correction : Remplacement de 'any' par 'unknown' (plus sûr pour un passage opaque)
  async generateCode(language: string, model: unknown): Promise<string> {
    try {
      // Appel à la commande Rust
      const result = await invoke<string>('generate_source_code', {
        language,
        model,
      });
      return result;
    } catch (error: unknown) {
      console.error('❌ Erreur génération de code:', error);
      throw error;
    }
  }
}

export const codegenService = new CodegenService();
