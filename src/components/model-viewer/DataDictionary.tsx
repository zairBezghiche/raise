import { useState } from 'react';
import { useModelStore } from '@/store/model-store';

interface NamedElement {
  id?: string;
  uuid?: string;
  name?: string;
  description?: string;
  [key: string]: unknown;
}

interface LayerSectionProps {
  title: string;
  color: string;
  isOpen: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

export function DataDictionary() {
  const { project } = useModelStore();

  const [openLayers, setOpenLayers] = useState<Record<string, boolean>>({
    oa: true,
    sa: true,
    la: true,
    pa: true,
    epbs: true,
    data: true,
  });

  if (!project) {
    return (
      <div style={{ padding: 'var(--spacing-4)', color: 'var(--text-muted)', fontStyle: 'italic' }}>
        Aucun modèle chargé.
      </div>
    );
  }

  const toggleLayer = (layer: string) => {
    setOpenLayers((prev) => ({ ...prev, [layer]: !prev[layer] }));
  };

  // Helper pour afficher une liste d'éléments
  const renderElementList = (title: string, elements: NamedElement[] | undefined, icon: string) => {
    if (!elements || elements.length === 0) return null;
    return (
      <div style={{ marginLeft: 'var(--spacing-4)', marginBottom: 'var(--spacing-2)' }}>
        <h4
          style={{
            fontSize: 'var(--font-size-xs)',
            color: 'var(--text-muted)',
            marginBottom: 'var(--spacing-2)',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
          }}
        >
          {title} <span style={{ opacity: 0.7 }}>({elements.length})</span>
        </h4>
        <div style={{ display: 'grid', gap: 'var(--spacing-2)' }}>
          {elements.map((el, idx) => (
            <div
              key={el.id || el.uuid || idx}
              style={{
                backgroundColor: 'var(--bg-app)', // Fond contrasté par rapport au panel
                padding: '8px 12px',
                borderRadius: 'var(--radius-sm)',
                borderLeft: '3px solid var(--color-primary)',
                display: 'flex',
                alignItems: 'center',
                gap: 'var(--spacing-2)',
                fontSize: 'var(--font-size-sm)',
                color: 'var(--text-main)',
                boxShadow: 'var(--shadow-sm)',
              }}
            >
              <span>{icon}</span>
              <strong style={{ fontWeight: 'var(--font-weight-medium)' }}>
                {el.name || 'Sans Nom'}
              </strong>
              {el.description && (
                <span style={{ color: 'var(--text-muted)', fontSize: '0.9em', marginLeft: 'auto' }}>
                  — {el.description.substring(0, 40)}...
                </span>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };

  // Cast sécurisé pour accéder aux couches dynamiques
  const p = project as Record<string, unknown>;
  const meta = (p.meta || {}) as Record<string, string>;

  // Helpers d'extraction typés
  const getLayer = (key: string) => (p[key] || {}) as Record<string, NamedElement[]>;
  const checkLayer = (layerData: Record<string, unknown>) =>
    layerData && Object.values(layerData).some((arr) => Array.isArray(arr) && arr.length > 0);

  const oa = getLayer('oa');
  const sa = getLayer('sa');
  const la = getLayer('la');
  const pa = getLayer('pa');
  const epbs = getLayer('epbs');
  const data = getLayer('data');

  const hasOA = checkLayer(oa);
  const hasSA = checkLayer(sa);
  const hasLA = checkLayer(la);
  const hasPA = checkLayer(pa);
  const hasEPBS = checkLayer(epbs);
  const hasData = checkLayer(data);

  return (
    <div
      style={{
        padding: 'var(--spacing-4)',
        overflowY: 'auto',
        height: '100%',
        fontFamily: 'var(--font-family)',
      }}
    >
      <header
        style={{
          marginBottom: 'var(--spacing-6)',
          borderBottom: '1px solid var(--border-color)',
          paddingBottom: 'var(--spacing-4)',
        }}
      >
        <h2 style={{ margin: 0, color: 'var(--color-primary)', fontSize: 'var(--font-size-xl)' }}>
          Explorateur de Modèle
        </h2>
        <div
          style={{
            color: 'var(--text-muted)',
            marginTop: 'var(--spacing-2)',
            fontSize: 'var(--font-size-sm)',
          }}
        >
          Projet :{' '}
          <strong style={{ color: 'var(--text-main)' }}>
            {(typeof p.name === 'string' ? p.name : meta.name) || 'Inconnu'}
          </strong>
          <span style={{ margin: '0 10px' }}>•</span>
          ID:{' '}
          <code
            style={{
              backgroundColor: 'var(--bg-app)',
              padding: '2px 6px',
              borderRadius: '4px',
              fontFamily: 'var(--font-family-mono)',
            }}
          >
            {typeof p.id === 'string' ? p.id : 'N/A'}
          </code>
        </div>
      </header>

      {/* Sections dynamiques */}
      {hasOA && (
        <LayerSection
          title="🌍 Analyse Opérationnelle (OA)"
          color="#f59e0b"
          isOpen={openLayers.oa}
          onToggle={() => toggleLayer('oa')}
        >
          {renderElementList('Acteurs', oa.actors, '👤')}
          {renderElementList('Activités', oa.activities, '⚙️')}
        </LayerSection>
      )}

      {hasSA && (
        <LayerSection
          title="🔭 Analyse Système (SA)"
          color="#10b981"
          isOpen={openLayers.sa}
          onToggle={() => toggleLayer('sa')}
        >
          {renderElementList('Acteurs', sa.actors, '👤')}
          {renderElementList('Fonctions', sa.functions, 'ƒ')}
        </LayerSection>
      )}

      {hasLA && (
        <LayerSection
          title="🧠 Architecture Logique (LA)"
          color="#3b82f6"
          isOpen={openLayers.la}
          onToggle={() => toggleLayer('la')}
        >
          {renderElementList('Composants', la.components, '📦')}
          {renderElementList('Fonctions', la.functions, 'ƒ')}
        </LayerSection>
      )}

      {hasPA && (
        <LayerSection
          title="⚙️ Architecture Physique (PA)"
          color="#8b5cf6"
          isOpen={openLayers.pa}
          onToggle={() => toggleLayer('pa')}
        >
          {renderElementList('Composants (Node)', pa.components, '🖥️')}
        </LayerSection>
      )}

      {hasEPBS && (
        <LayerSection
          title="📦 End Product Breakdown (EPBS)"
          color="#db2777"
          isOpen={openLayers.epbs}
          onToggle={() => toggleLayer('epbs')}
        >
          {renderElementList('Configuration Items (CI)', epbs.configurationItems, '🍱')}
        </LayerSection>
      )}

      <LayerSection
        title="📚 Dictionnaire de Données"
        color="var(--color-gray-500)"
        isOpen={openLayers.data}
        onToggle={() => toggleLayer('data')}
      >
        {hasData ? (
          <>
            {renderElementList('Classes', data.classes, '🏷️')}
            {renderElementList('Types', data.dataTypes, '🔢')}
          </>
        ) : (
          <div style={{ padding: '10px', fontStyle: 'italic', color: 'var(--text-muted)' }}>
            Vide
          </div>
        )}
      </LayerSection>
    </div>
  );
}

// Composant interne pour l'en-tête de section
function LayerSection({ title, color, isOpen, onToggle, children }: LayerSectionProps) {
  return (
    <div style={{ marginBottom: 'var(--spacing-6)' }}>
      <div
        onClick={onToggle}
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: 'var(--spacing-3)',
          backgroundColor: 'var(--bg-panel)', // Adaptatif
          borderLeft: `5px solid ${color}`,
          border: '1px solid var(--border-color)',
          borderLeftWidth: '5px', // Priorité sur le border global
          borderLeftColor: color,
          borderRadius: 'var(--radius-md)',
          cursor: 'pointer',
          fontWeight: 'var(--font-weight-bold)',
          color: 'var(--text-main)',
          marginBottom: 'var(--spacing-2)',
          boxShadow: 'var(--shadow-sm)',
          transition: 'var(--transition-fast)',
        }}
      >
        <span>{title}</span>
        <span style={{ fontSize: '0.8em', color: 'var(--text-muted)' }}>{isOpen ? '▼' : '▶'}</span>
      </div>
      {isOpen && (
        <div
          style={{
            paddingLeft: 'var(--spacing-4)',
            borderLeft: '1px dashed var(--border-color)',
            marginLeft: 'var(--spacing-4)',
          }}
        >
          {children}
        </div>
      )}
    </div>
  );
}
