// Correction de l'import : on pointe vers types/ai.types.ts
import type { ChatMessage } from '@/types/ai.types';

interface IntentClassifierProps {
  lastMessage?: ChatMessage;
}

function guessIntent(text: string): string {
  const lower = text.toLowerCase();
  if (lower.includes('capella') || lower.includes('arcadia')) return 'Modélisation système';
  if (lower.includes('pipeline') || lower.includes('ci/cd')) return 'DevOps / CI-CD';
  if (lower.includes('schema') || lower.includes('json')) return 'Schémas / données';
  return 'Général';
}

export function IntentClassifier({ lastMessage }: IntentClassifierProps) {
  if (!lastMessage) return null;
  if (lastMessage.role !== 'user') return null;

  const intent = guessIntent(lastMessage.content);

  return (
    <div
      style={{
        fontSize: 'var(--font-size-xs)',
        color: 'var(--color-primary)',
        fontWeight: 'var(--font-weight-medium)',
        marginBottom: 'var(--spacing-2)',
        backgroundColor: 'var(--color-gray-50)',
        display: 'inline-block',
        padding: '2px 8px',
        borderRadius: 'var(--radius-sm)',
      }}
    >
      Intent détectée : {intent}
    </div>
  );
}
