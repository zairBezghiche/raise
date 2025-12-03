// FICHIER : src/types/model.types.ts

export interface I18nString {
  [lang: string]: string;
}

export interface ArcadiaElement {
  id: string;
  name: string | I18nString;
  type: string; // URI complète (ex: https://...#OperationalActor)
  // Propriétés dynamiques (PVMT, attributs métier)
  [key: string]: any;
}

// --- Couches ---

export interface OperationalAnalysisLayer {
  actors: ArcadiaElement[];
  activities: ArcadiaElement[];
  capabilities: ArcadiaElement[];
  entities: ArcadiaElement[];
  exchanges: ArcadiaElement[];
}

export interface SystemAnalysisLayer {
  components: ArcadiaElement[];
  actors: ArcadiaElement[];
  functions: ArcadiaElement[];
  capabilities: ArcadiaElement[];
  exchanges: ArcadiaElement[];
}

export interface LogicalArchitectureLayer {
  components: ArcadiaElement[];
  actors: ArcadiaElement[];
  functions: ArcadiaElement[];
  interfaces: ArcadiaElement[];
  exchanges: ArcadiaElement[];
}

export interface PhysicalArchitectureLayer {
  components: ArcadiaElement[];
  actors: ArcadiaElement[];
  functions: ArcadiaElement[];
  links: ArcadiaElement[];
  exchanges: ArcadiaElement[];
}

export interface EPBSLayer {
  configurationItems: ArcadiaElement[];
}

export interface ProjectMeta {
  name: string;
  loadedAt: string;
  elementCount: number;
}

// --- Racine ---

export interface ProjectModel {
  oa: OperationalAnalysisLayer;
  sa: SystemAnalysisLayer;
  la: LogicalArchitectureLayer;
  pa: PhysicalArchitectureLayer;
  epbs: EPBSLayer;
  meta: ProjectMeta;
}
