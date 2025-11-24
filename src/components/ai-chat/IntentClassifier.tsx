import type { ChatMessage } from '@/store/ai-store'

interface IntentClassifierProps {
  lastMessage?: ChatMessage
}

function guessIntent(text: string): string {
  const lower = text.toLowerCase()
  if (lower.includes('capella') || lower.includes('arcadia')) return 'Modélisation système'
  if (lower.includes('pipeline') || lower.includes('ci/cd')) return 'DevOps / CI-CD'
  if (lower.includes('schema') || lower.includes('json')) return 'Schémas / données'
  return 'Général'
}

export function IntentClassifier({ lastMessage }: IntentClassifierProps) {
  if (!lastMessage) return null
  if (lastMessage.role !== 'user') return null

  const intent = guessIntent(lastMessage.content)

  return (
    <div
      style={{
        fontSize: 11,
        color: '#9ca3af',
        marginBottom: 4,
      }}
    >
      Intent détectée : <strong>{intent}</strong>
    </div>
  )
}
