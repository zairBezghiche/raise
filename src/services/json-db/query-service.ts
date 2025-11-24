/**
 * Service de construction + exécution de requêtes JSON-DB
 * vers les commandes Tauri `jsondb_query_collection`.
 */

import { invoke } from '@tauri-apps/api/core'

export type LogicalOperator = 'and' | 'or' | 'not'
export type ComparisonOperator =
  | 'eq'
  | 'ne'
  | 'gt'
  | 'gte'
  | 'lt'
  | 'lte'
  | 'in'
  | 'contains'
  | 'startsWith'
  | 'endsWith'

export type SortOrder = 'asc' | 'desc'

/**
 * Forme JSON envoyée côté Rust (doit matcher QueryFilter).
 *
 * - Comparaison simple : { op: "eq", field: "handle", value: "..." }
 * - AND / OR :          { op: "and", filters: [ ... ] }
 * - NOT :               { op: "not", filter: { ... } }
 */
export type QueryFilter =
  | {
      op: ComparisonOperator
      field: string
      value: unknown
    }
  | {
      op: 'and' | 'or'
      filters: QueryFilter[]
    }
  | {
      op: 'not'
      filter: QueryFilter
    }

export interface SortField {
  field: string
  order: SortOrder
}

export interface Query {
  collection: string
  filter?: QueryFilter
  sort?: SortField[]
  offset?: number
  limit?: number
}

/**
 * Fluent builder pour construire un Query proprement.
 */
export class QueryBuilder {
  private readonly collectionName: string
  private currentFilter?: QueryFilter
  private sortFields: SortField[] = []
  private offsetValue?: number
  private limitValue?: number

  constructor(collection: string) {
    this.collectionName = collection
  }

  /** Comparaison simple : field op value (eq, ne, gt, ...) */
  where(
    field: string,
    op: ComparisonOperator,
    value: unknown,
  ): this {
    const cond: QueryFilter = { op, field, value }
    if (!this.currentFilter) {
      this.currentFilter = cond
    } else {
      this.currentFilter = {
        op: 'and',
        filters: [this.currentFilter, cond],
      }
    }
    return this
  }

  /** Ajoute un AND explicite */
  and(filter: QueryFilter): this {
    if (!this.currentFilter) {
      this.currentFilter = filter
    } else {
      this.currentFilter = {
        op: 'and',
        filters: [this.currentFilter, filter],
      }
    }
    return this
  }

  /** Ajoute un OR explicite */
  or(filter: QueryFilter): this {
    if (!this.currentFilter) {
      this.currentFilter = filter
    } else {
      this.currentFilter = {
        op: 'or',
        filters: [this.currentFilter, filter],
      }
    }
    return this
  }

  /** NOT sur le filtre courant (ou sur un filtre donné) */
  not(filter?: QueryFilter): this {
    const target = filter ?? this.currentFilter
    if (!target) return this
    this.currentFilter = {
      op: 'not',
      filter: target,
    }
    return this
  }

  /** Ajoute un tri : ex orderBy("createdAt", "desc") */
  orderBy(field: string, order: SortOrder = 'asc'): this {
    this.sortFields.push({ field, order })
    return this
  }

  limit(n: number): this {
    this.limitValue = n
    return this
  }

  offset(n: number): this {
    this.offsetValue = n
    return this
  }

  build(): Query {
    return {
      collection: this.collectionName,
      filter: this.currentFilter,
      sort: this.sortFields.length ? this.sortFields : undefined,
      offset: this.offsetValue,
      limit: this.limitValue,
    }
  }
}

/**
 * Service d’appel Tauri sur la DB JSON.
 */
export class JsonDbQueryService {
  constructor(
    private readonly space: string,
    private readonly db: string,
  ) {}

  /**
   * Exécute une requête via la commande Tauri `jsondb_query_collection`.
   * latest = true applique le raccourci (tri createdAt:desc + limit=1 côté Rust).
   */
  async execute(
    query: Query,
    options?: { latest?: boolean },
  ): Promise<unknown[]> {
    const { collection, filter, sort, limit, offset } = query

    const filterJson = filter ? JSON.stringify(filter) : undefined
    const sortSpec = sort?.map((s) => `${s.field}:${s.order}`) ?? []

    const results = await invoke<unknown[]>('jsondb_query_collection', {
      space: this.space,
      db: this.db,
      collection,
      filterJson,
      sort: sortSpec,
      limit: limit ?? null,
      latest: options?.latest ?? false,
      offset: offset ?? null,
    })

    return results
  }
}

/** Helper pratique côté UI */
export const createQuery = (collection: string) =>
  new QueryBuilder(collection)
