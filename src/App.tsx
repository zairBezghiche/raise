import { useEffect } from 'react'
import { SchemaService } from '@/services/json-db/schema-service'

export default function App() {
  useEffect(() => {
    const svc = new SchemaService()
    ;(async () => {
      await svc.registerSchema('demo', { $id: 'demo', type: 'object' })
      const s = await svc.getSchema('demo')
      console.log('schema demo =', s)
    })().catch(console.error)
  }, [])

  return <h1>GenAptitude</h1>
}
