import { create } from 'zustand'

export type ArcadiaLayer = 'OA' | 'SA' | 'LA' | 'PA'

export interface ModelElement {
  id: string
  name: string
  type: string
  layer?: ArcadiaLayer
  parentId?: string | null
  childrenIds?: string[]
  metadata?: Record<string, unknown>
}

export interface ModelStoreState {
  currentModelId?: string
  elementsById: Record<string, ModelElement>
  selectedElementId?: string

  setCurrentModel: (id: string | undefined) => void
  setElements: (elements: ModelElement[]) => void
  selectElement: (id: string | undefined) => void
}

export const useModelStore = create<ModelStoreState>((set) => ({
  currentModelId: undefined,
  elementsById: {},
  selectedElementId: undefined,

  setCurrentModel: (id) => set({ currentModelId: id }),

  setElements: (elements) =>
    set(() => {
      const map: Record<string, ModelElement> = {}
      for (const el of elements) {
        map[el.id] = el
      }
      return { elementsById: map }
    }),

  selectElement: (id) => set({ selectedElementId: id }),
}))
