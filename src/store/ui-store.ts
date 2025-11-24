import { create } from 'zustand'

export type ThemeMode = 'light' | 'dark' | 'system'

export interface UiStoreState {
  theme: ThemeMode
  sidebarOpen: boolean
  panelLayout: 'single' | 'split'

  setTheme: (mode: ThemeMode) => void
  toggleSidebar: () => void
  setPanelLayout: (layout: 'single' | 'split') => void
}

export const useUiStore = create<UiStoreState>((set) => ({
  theme: 'system',
  sidebarOpen: true,
  panelLayout: 'split',

  setTheme: (mode) => set({ theme: mode }),
  toggleSidebar: () =>
    set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  setPanelLayout: (layout) => set({ panelLayout: layout }),
}))
