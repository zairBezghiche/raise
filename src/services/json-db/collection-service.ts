import { invoke } from '@tauri-apps/api/core';
import type { Query } from './query-service';

const DEFAULT_SPACE = 'un2';
const DEFAULT_DB = '_system';

export interface Document<T = any> {
  id: string;
  [key: string]: T | any;
}

export interface QueryResult<T = any> {
  documents: T[];
  total_count: number;
  offset: number;
  limit: number | null;
}

export class CollectionService {
  /**
   * Crée une collection avec un schéma spécifique.
   * @param name Nom de la collection
   * @param schema Chemin relatif du schéma (ex: "sandbox/generic.schema.json")
   */
  async createCollection(name: string, schema?: string): Promise<void> {
    await invoke('jsondb_create_collection', {
      space: DEFAULT_SPACE,
      db: DEFAULT_DB,
      collection: name,
      schema: schema ?? 'sandbox/generic.schema.json',
    });
  }

  async insertRaw(collection: string, doc: any): Promise<void> {
    await invoke('jsondb_insert_raw', {
      space: DEFAULT_SPACE,
      db: DEFAULT_DB,
      collection,
      doc,
    });
  }

  async listAll(collection: string): Promise<Document[]> {
    return await invoke<Document[]>('jsondb_list_all', {
      space: DEFAULT_SPACE,
      db: DEFAULT_DB,
      collection,
    });
  }

  /**
   * Exécute une requête complexe via le moteur de recherche backend.
   * @param collection Nom de la collection
   * @param query Objet Query construit via createQuery()
   */
  async queryDocuments(collection: string, query: Query): Promise<Document[]> {
    // On s'assure que le champ collection est bien rempli dans l'objet Query
    const queryObj = { ...query, collection };

    // Appel de la commande Rust
    // Note : le paramètre '_bucket' est un placeholder requis par la signature Rust actuelle
    const result = await invoke<QueryResult>('jsondb_query_collection', {
      space: DEFAULT_SPACE,
      db: DEFAULT_DB,
      _bucket: collection,
      queryJson: JSON.stringify(queryObj),
    });

    return result.documents;
  }
}

export const collectionService = new CollectionService();
