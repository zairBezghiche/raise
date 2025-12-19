import { useState } from 'react';
import { useAIChat } from '@/hooks/useAIChat';
import { MessageBubble } from './MessageBubble';
import { InputBar } from './InputBar';
import { SuggestionPanel } from './SuggestionPanel';
import { ContextDisplay } from './ContextDisplay';
import { IntentClassifier } from './IntentClassifier';
import { CreatedArtifact } from '@/types/ai.types'; // Import nécessaire

export function ChatInterface() {
  const { messages, isThinking, error, sendMessage, clear } = useAIChat();
  const [input, setInput] = useState('');

  const lastMessage = messages[messages.length - 1];

  function handleSend(value: string) {
    setInput('');
    void sendMessage(value);
  }

  // --- NOUVEAU : Fonction intelligente de génération de code ---
  const handleGenerateCode = (language: string, artifact: CreatedArtifact) => {
    // On construit un prompt contextuel puissant
    const prompt = `Agis en tant qu'expert Software. 
    Génère le code ${language.toUpperCase()} complet pour l'élément "${artifact.name}" (Type: ${
      artifact.element_type
    }, Layer: ${artifact.layer}).
    Inclus :
    - Les imports nécessaires
    - La documentation (commentaires)
    - Une implémentation robuste correspondant à la définition JSON précédente.`;

    // On envoie ce message comme si c'était l'utilisateur
    void sendMessage(prompt);
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        maxHeight: '100vh',
        backgroundColor: 'var(--bg-panel)',
        color: 'var(--text-main)',
        padding: 'var(--spacing-4)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--border-color)',
        transition: 'var(--transition-base)',
      }}
    >
      <header
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 'var(--spacing-4)',
        }}
      >
        <div>
          <h2 style={{ fontSize: 'var(--font-size-lg)', margin: 0, color: 'var(--text-main)' }}>
            Assistant GenAptitude
          </h2>
          <ContextDisplay messagesCount={messages.length} />
        </div>
        <button
          type="button"
          onClick={clear}
          style={{
            fontSize: 'var(--font-size-xs)',
            borderRadius: 'var(--radius-full)',
            border: '1px solid var(--border-color)',
            backgroundColor: 'var(--color-gray-50)',
            color: 'var(--text-muted)',
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
          'Défini la classe Client avec nom et prénom',
          'Ajoute une fonction "Calculer Prix" (SA)',
        ]}
        onSelect={setInput}
      />

      <div style={{ flex: 1, overflowY: 'auto', padding: 'var(--spacing-2) 0' }}>
        {messages.map((m) => (
          <MessageBubble
            key={m.id}
            message={m}
            // On passe notre nouvelle fonction ici !
            onGenerateCode={handleGenerateCode}
          />
        ))}

        {isThinking && (
          <div
            style={{
              fontSize: 'var(--font-size-xs)',
              color: 'var(--text-muted)',
              marginTop: 'var(--spacing-2)',
              fontStyle: 'italic',
            }}
          >
            L’assistant réfléchit…
          </div>
        )}

        {error && (
          <div
            style={{
              fontSize: 'var(--font-size-xs)',
              color: 'var(--color-error)',
              marginTop: 'var(--spacing-2)',
            }}
          >
            Erreur : {error}
          </div>
        )}
      </div>

      <InputBar value={input} onChange={setInput} onSend={handleSend} disabled={isThinking} />
    </div>
  );
}
