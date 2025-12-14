// FICHIER : src/types/json-db.types.ts

// --- Query Engine ---

export type SortOrder = 'Asc' | 'Desc';

export interface SortField {
  field: string;
  order: SortOrder;
}

export type FilterOperator = 'And' | 'Or' | 'Not';

export type ComparisonOperator =
  | 'Eq'
  | 'Ne'
  | 'Gt'
  | 'Gte'
  | 'Lt'
  | 'Lte'
  | 'In'
  | 'Contains'
  | 'StartsWith'
  | 'EndsWith'
  | 'Matches';

export interface Condition {
  field: string;
  operator: ComparisonOperator;
  // CORRECTION : unknown accepte tout type (string, number, date...) de manière sécurisée
  value: unknown;
}

export interface QueryFilter {
  operator: FilterOperator;
  conditions: Condition[];
}

export interface Query {
  collection: string;
  filter?: QueryFilter;
  sort?: SortField[];
  limit?: number;
  offset?: number;
  projection?: string[];
}

export interface QueryResponse {
  // CORRECTION : Typage fort avec l'interface Document définie plus bas
  documents: Document[];
  total: number;
}

// --- Transactions ---

export type OperationRequest =
  // CORRECTION : Record<string, unknown> pour un objet JSON générique
  | { type: 'Insert'; collection: string; id: string; document: Record<string, unknown> }
  | { type: 'Update'; collection: string; id: string; document: Record<string, unknown> }
  | { type: 'Delete'; collection: string; id: string };

export interface TransactionRequest {
  operations: OperationRequest[];
}

// --- Document Générique ---

// CORRECTION : Interface simplifiée et sécurisée sans 'any'
// L'index signature [key: string]: unknown permet d'ajouter n'importe quelle propriété
// mais obligera à vérifier son type avant usage (type narrowing).
export interface Document {
  id: string;
  [key: string]: unknown;
}
