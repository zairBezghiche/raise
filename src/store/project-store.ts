import { create } from 'zustand'

export interface Project {
  id: string
  name: string
  path?: string
  domain?: string // ex: "software", "system", "hardware"
}

export interface ProjectStoreState {
  projects: Project[]
  activeProjectId?: string

  addProject: (p: Project) => void
  removeProject: (id: string) => void
  setActiveProject: (id: string | undefined) => void
}

export const useProjectStore = create<ProjectStoreState>((set) => ({
  projects: [],
  activeProjectId: undefined,

  addProject: (p) =>
    set((state) => ({
      projects: [...state.projects, p],
    })),

  removeProject: (id) =>
    set((state) => ({
      projects: state.projects.filter((p) => p.id !== id),
      activeProjectId:
        state.activeProjectId === id ? undefined : state.activeProjectId,
    })),

  setActiveProject: (id) => set({ activeProjectId: id }),
}))
