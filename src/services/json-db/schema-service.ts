import { invoke } from '@tauri-apps/api/core'

export type JsonSchema = unknown

export class SchemaService {
  async registerSchema(schemaId: string, schema: JsonSchema): Promise<void> {
    await invoke('register_schema', {
      schemaId,
      schemaJson: JSON.stringify(schema, null, 2),
    })
  }

  async getSchema(schemaId: string): Promise<JsonSchema> {
    const text = await invoke<string>('get_schema', { schemaId })
    return JSON.parse(text)
  }
}
