/**
 * Service de gestion JSON-LD pour données liées.
 *
 * ⚠️ Version simplifiée :
 * - on ne fait pas de vraie expansion/compaction (pas de lib jsonld)
 * - on gère des contextes nommés et on ajoute/enlève @context proprement
 */

export interface JsonLdContext {
  '@context': ContextDefinition
}

export type ContextDefinition =
  | string
  | Record<string, ContextValue>
  | ContextDefinition[]

export type ContextValue =
  | string
  | {
      '@id': string
      '@type'?: string
      '@container'?: string
    }

export class JsonLdService {
  private readonly contexts = new Map<string, JsonLdContext>()

  /**
   * Enregistre un contexte nommé.
   * ex: registerContext("arcadia", { "@context": { "oa:Actor": "...", ... } })
   */
  registerContext(name: string, context: JsonLdContext): void {
    this.contexts.set(name, context)
  }

  getContext(name: string): JsonLdContext | undefined {
    return this.contexts.get(name)
  }

  /**
   * "Expand" au sens GenAptitude :
   * - ajoute @context au document
   * - conserve toutes les propriétés telles quelles
   */
  expandDocument(document: any, contextName: string): any {
    const context = this.getContext(contextName)
    if (!context) {
      throw new Error(`JSON-LD context not found: ${contextName}`)
    }
    return {
      '@context': context['@context'],
      ...document,
    }
  }

  /**
   * Compaction simplifiée :
   * - retire simplement la clé @context
   * - renvoie un document "nu" JSON classique
   */
  compactDocument(document: any): any {
    const { '@context': _ctx, ...data } = document ?? {}
    return data
  }
}

export const jsonLdService = new JsonLdService()
