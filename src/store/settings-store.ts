import { create } from 'zustand'

export type AiBackend = 'mock' | 'tauri-local' | 'remote-api'

export interface SettingsState {
  language: 'fr' | 'en'
  aiBackend: AiBackend
  jsonDbSpace: string
  jsonDbDatabase: string

  update: (partial: Partial<SettingsState>) => void
}

export const useSettingsStore = create<SettingsState>((set) => ({
  language: 'fr',
  aiBackend: 'mock',
  jsonDbSpace: 'un2',
  jsonDbDatabase: '_system',

  update: (partial) => set((state) => ({ ...state, ...partial })),
}))
