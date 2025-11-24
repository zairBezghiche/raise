import type { ChatMessage } from '@/store/ai-store'

interface MessageBubbleProps {
  message: ChatMessage
}

export function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === 'user'

  return (
    <div
      className="ga-chat-bubble"
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: isUser ? 'flex-end' : 'flex-start',
        marginBottom: 8,
      }}
    >
      <div
        style={{
          maxWidth: '80%',
          padding: '8px 12px',
          borderRadius: 12,
          backgroundColor: isUser ? '#4f46e5' : '#111827',
          color: '#f9fafb',
          fontSize: 14,
          whiteSpace: 'pre-wrap',
        }}
      >
        {message.content}
      </div>
      <div
        style={{
          fontSize: 11,
          color: '#6b7280',
          marginTop: 2,
        }}
      >
        {isUser ? 'Vous' : 'Assistant'} ·{' '}
        {new Date(message.createdAt).toLocaleTimeString()}
      </div>
    </div>
  )
}
