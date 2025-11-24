interface ContextDisplayProps {
    messagesCount: number
  }
  
  export function ContextDisplay({ messagesCount }: ContextDisplayProps) {
    return (
      <div
        style={{
          fontSize: 12,
          color: '#6b7280',
          marginBottom: 8,
        }}
      >
        Session de chat · {messagesCount} message(s)
      </div>
    )
  }
  