// FICHIER : src/store/model-store.ts

import { create } from 'zustand';
import type { ProjectModel, ArcadiaElement } from '@/types/model.types';

export interface ModelStoreState {
  // État
  project: ProjectModel | null;
  isLoading: boolean;
  error: string | null;

  // Indexation rapide
  elementsById: Record<string, ArcadiaElement>;

  // Actions
  setProject: (model: ProjectModel) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Helpers
  getElementById: (id: string) => ArcadiaElement | undefined;
}

export const useModelStore = create<ModelStoreState>((set, get) => ({
  project: null,
  isLoading: false,
  error: null,
  elementsById: {},

  setProject: (model) => {
    // On indexe tout à plat pour les recherches rapides O(1)
    const map: Record<string, ArcadiaElement> = {};

    const indexLayer = (elements: ArcadiaElement[]) => {
      elements.forEach((el) => {
        map[el.id] = el;
      });
    };

    if (model.oa) {
      indexLayer(model.oa.actors);
      indexLayer(model.oa.activities);
      // ... ajouter les autres listes si besoin
    }
    if (model.sa) {
      indexLayer(model.sa.functions);
      indexLayer(model.sa.components);
    }
    // ... LA, PA, EPBS

    set({ project: model, elementsById: map, error: null });
  },

  setLoading: (isLoading) => set({ isLoading }),

  setError: (error) => set({ error }),

  getElementById: (id) => get().elementsById[id],
}));
