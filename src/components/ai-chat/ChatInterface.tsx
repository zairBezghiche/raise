import { useState } from 'react'
import { useAIChat } from '@/hooks/useAIChat'
import { MessageBubble } from './MessageBubble'
import { InputBar } from './InputBar'
import { SuggestionPanel } from './SuggestionPanel'
import { ContextDisplay } from './ContextDisplay'
import { IntentClassifier } from './IntentClassifier'

export function ChatInterface() {
  const { messages, isThinking, error, sendMessage, clear } = useAIChat()
  const [input, setInput] = useState('')

  const lastMessage = messages[messages.length - 1]

  function handleSend(value: string) {
    setInput('')
    void sendMessage(value)
  }

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        maxHeight: '100vh',
        backgroundColor: '#020617',
        color: '#f9fafb',
        padding: 12,
        borderRadius: 12,
        border: '1px solid #111827',
      }}
    >
      <header
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 8,
        }}
      >
        <div>
          <h2 style={{ fontSize: 16, margin: 0 }}>Assistant GenAptitude</h2>
          <ContextDisplay messagesCount={messages.length} />
        </div>
        <button
          type="button"
          onClick={clear}
          style={{
            fontSize: 12,
            borderRadius: 999,
            border: '1px solid #374151',
            backgroundColor: '#020617',
            color: '#9ca3af',
            padding: '4px 10px',
            cursor: 'pointer',
          }}
        >
          Effacer
        </button>
      </header>

      <IntentClassifier lastMessage={lastMessage} />

      <SuggestionPanel
        suggestions={[
          'Explique-moi la structure JSON-DB actuelle',
          'Montre-moi un exemple de requête sur la collection "articles"',
          'Comment brancher Capella / Arcadia sur GenAptitude ?',
        ]}
        onSelect={setInput}
      />

      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: '8px 0',
        }}
      >
        {messages.map((m) => (
          <MessageBubble key={m.id} message={m} />
        ))}
        {isThinking && (
          <div style={{ fontSize: 12, color: '#9ca3af', marginTop: 8 }}>
            L’assistant réfléchit…
          </div>
        )}
        {error && (
          <div
            style={{
              fontSize: 12,
              color: '#f97373',
              marginTop: 8,
            }}
          >
            Erreur : {error}
          </div>
        )}
      </div>

      <InputBar
        value={input}
        onChange={setInput}
        onSend={handleSend}
        disabled={isThinking}
      />
    </div>
  )
}
