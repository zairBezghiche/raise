import { useState } from 'react';
import InvoiceDemo from './InvoiceDemo';
import ModelRulesDemo from './ModelRulesDemo';

type RulesTab = 'model' | 'invoice';

export default function RulesEngineDashboard() {
  const [activeTab, setActiveTab] = useState<RulesTab>('model');

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* --- MENU LOCAL (GAUCHE) --- */}
      <div
        style={{
          width: '260px',
          backgroundColor: 'var(--bg-panel)',
          borderRight: '1px solid var(--border-color)',
          padding: '20px',
          display: 'flex',
          flexDirection: 'column',
          gap: '10px',
        }}
      >
        <h3
          style={{
            fontSize: '0.75rem',
            fontWeight: 'bold',
            textTransform: 'uppercase',
            color: 'var(--text-muted)',
            marginBottom: '10px',
          }}
        >
          Scénarios
        </h3>

        <MenuButton
          active={activeTab === 'model'}
          onClick={() => setActiveTab('model')}
          label="📐 Règles Modèle"
          desc="Validation Ingénierie"
        />

        <MenuButton
          active={activeTab === 'invoice'}
          onClick={() => setActiveTab('invoice')}
          label="🧾 Facturation"
          desc="Calculs & Lookup DB"
        />
      </div>

      {/* --- CONTENU (DROITE) --- */}
      <div style={{ flex: 1, overflowY: 'auto', backgroundColor: 'var(--bg-app)' }}>
        <div style={{ padding: '20px', maxWidth: '1000px', margin: '0 auto' }}>
          {activeTab === 'model' ? <ModelRulesDemo /> : <InvoiceDemo />}
        </div>
      </div>
    </div>
  );
}

// Petit composant helper pour les boutons du menu
function MenuButton({
  active,
  onClick,
  label,
  desc,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  desc: string;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        textAlign: 'left',
        padding: '12px',
        borderRadius: '8px',
        backgroundColor: active ? 'rgba(var(--color-primary-rgb), 0.1)' : 'transparent',
        border: active ? '1px solid var(--color-primary)' : '1px solid transparent',
        cursor: 'pointer',
        transition: 'all 0.2s',
      }}
    >
      <div
        style={{
          color: active ? 'var(--color-primary)' : 'var(--text-main)',
          fontWeight: 'bold',
          fontSize: '0.9rem',
        }}
      >
        {label}
      </div>
      <div
        style={{
          color: 'var(--text-muted)',
          fontSize: '0.75rem',
          marginTop: '4px',
        }}
      >
        {desc}
      </div>
    </button>
  );
}
