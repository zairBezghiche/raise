import { useState } from 'react';
import { CreatedArtifact } from '@/types/ai.types';

interface ArtifactCardProps {
  artifact: CreatedArtifact;
  onClick?: (path: string) => void;
  // NOUVEAU : Callback pour l'action de génération
  onGenerateCode?: (language: 'sql' | 'rust' | 'python', artifact: CreatedArtifact) => void;
}

export function ArtifactCard({ artifact, onClick, onGenerateCode }: ArtifactCardProps) {
  const [isHovered, setIsHovered] = useState(false);
  const [showActions, setShowActions] = useState(false);

  // Code couleur par couche Arcadia
  const layerColors: Record<string, string> = {
    OA: '#eab308',
    SA: '#a855f7',
    LA: '#3b82f6',
    PA: '#22c55e',
    EPBS: '#f97316',
    DATA: '#ef4444',
    TRANSVERSE: '#64748b',
  };

  const color = layerColors[artifact.layer] || '#64748b';

  return (
    <div
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => {
        setIsHovered(false);
        setShowActions(false);
      }}
      style={{
        display: 'flex',
        flexDirection: 'column', // Changement pour accueillir les boutons en dessous
        marginTop: '8px',
        backgroundColor: '#ffffff',
        border: `1px solid ${isHovered ? color : '#e2e8f0'}`,
        borderLeft: `4px solid ${color}`,
        borderRadius: '6px',
        boxShadow: isHovered ? '0 4px 6px -1px rgba(0, 0, 0, 0.1)' : 'none',
        transition: 'all 0.2s ease',
        maxWidth: '100%',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      {/* Partie Principale (Clickable pour naviguer) */}
      <div
        onClick={() => onClick?.(artifact.path)}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '12px',
          padding: '10px 14px',
          cursor: 'pointer',
        }}
      >
        {/* Badge Layer */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: `${color}20`,
            color: color,
            fontWeight: 'bold',
            fontSize: '0.7rem',
            width: '32px',
            height: '32px',
            borderRadius: '4px',
          }}
        >
          {artifact.layer}
        </div>

        {/* Info */}
        <div style={{ flex: 1, overflow: 'hidden' }}>
          <div style={{ fontWeight: 600, fontSize: '0.9rem', color: '#1e293b' }}>
            {artifact.name}
          </div>
          <div style={{ fontSize: '0.75rem', color: '#64748b' }}>{artifact.element_type}</div>
        </div>

        {/* Bouton "..." pour ouvrir les actions */}
        <div
          onClick={(e) => {
            e.stopPropagation();
            setShowActions(!showActions);
          }}
          style={{ color: '#94a3b8', padding: '4px', borderRadius: '4px', cursor: 'pointer' }}
        >
          ⋮
        </div>
      </div>

      {/* Barre d'Actions (Visible si hover ou clic menu) */}
      {(showActions || (isHovered && artifact.layer === 'DATA')) && (
        <div
          style={{
            display: 'flex',
            gap: '8px',
            padding: '8px 14px',
            backgroundColor: '#f8fafc',
            borderTop: '1px solid #e2e8f0',
          }}
        >
          <ActionButton label="SQL" onClick={() => onGenerateCode?.('sql', artifact)} />
          <ActionButton label="Rust" onClick={() => onGenerateCode?.('rust', artifact)} />
          <ActionButton label="Python" onClick={() => onGenerateCode?.('python', artifact)} />
        </div>
      )}
    </div>
  );
}

function ActionButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      style={{
        fontSize: '0.7rem',
        padding: '4px 8px',
        borderRadius: '4px',
        border: '1px solid #cbd5e1',
        backgroundColor: 'white',
        cursor: 'pointer',
        fontWeight: 600,
        color: '#475569',
      }}
    >
      Générer {label}
    </button>
  );
}
