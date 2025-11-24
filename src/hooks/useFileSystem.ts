// src/hooks/useFileSystem.ts
import { readTextFile, writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'

/**
 * Petit helper pour lire/écrire des JSON dans le FS Tauri 2
 * via le plugin @tauri-apps/plugin-fs.
 */
export function useFileSystem(
  baseDir: BaseDirectory = BaseDirectory.AppLocalData,
) {
  async function readJson<T = unknown>(path: string): Promise<T> {
    const text = await readTextFile(path, { baseDir })
    return JSON.parse(text) as T
  }

  async function writeJson(path: string, data: unknown): Promise<void> {
    const text = JSON.stringify(data, null, 2)
    await writeTextFile(path, text, { baseDir })
  }

  return {
    readJson,
    writeJson,
  }
}
