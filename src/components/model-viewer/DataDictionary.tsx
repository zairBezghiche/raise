import { useState } from 'react';
import { useModelStore } from '@/store/model-store';

interface NamedElement {
  id?: string;
  uuid?: string;
  name?: string;
  description?: string;
  [key: string]: any;
}

export function DataDictionary() {
  const { project } = useModelStore();

  // 1. Ajout de l'état 'epbs'
  const [openLayers, setOpenLayers] = useState<Record<string, boolean>>({
    oa: true,
    sa: true,
    la: true,
    pa: true,
    epbs: true,
    data: true,
  });

  if (!project) {
    return <div style={{ color: '#9ca3af', padding: 20 }}>Aucun modèle chargé.</div>;
  }

  const toggleLayer = (layer: string) => {
    setOpenLayers((prev) => ({ ...prev, [layer]: !prev[layer] }));
  };

  const renderElementList = (title: string, elements: NamedElement[] | undefined, icon: string) => {
    if (!elements || elements.length === 0) return null;
    return (
      <div style={{ marginLeft: 20, marginBottom: 10 }}>
        <h4
          style={{
            fontSize: '0.9em',
            color: '#9ca3af',
            marginBottom: 5,
            textTransform: 'uppercase',
          }}
        >
          {title} <span style={{ opacity: 0.5 }}>({elements.length})</span>
        </h4>
        <div style={{ display: 'grid', gap: 6 }}>
          {elements.map((el, idx) => (
            <div
              key={el.id || el.uuid || idx}
              style={{
                background: 'var(--surface-secondary, #1f2937)',
                padding: '8px 12px',
                borderRadius: 4,
                borderLeft: '3px solid var(--color-primary, #4f46e5)',
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                fontSize: '0.9em',
              }}
            >
              <span>{icon}</span>
              <strong>{el.name || 'Sans Nom'}</strong>
              {el.description && (
                <span style={{ color: '#6b7280', fontSize: '0.85em' }}>
                  — {el.description.substring(0, 50)}...
                </span>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };

  const p = project as any;

  // Détection des couches
  const hasOA =
    p.oa && Object.values(p.oa).some((arr: any) => Array.isArray(arr) && arr.length > 0);
  const hasSA =
    p.sa && Object.values(p.sa).some((arr: any) => Array.isArray(arr) && arr.length > 0);
  const hasLA =
    p.la && Object.values(p.la).some((arr: any) => Array.isArray(arr) && arr.length > 0);
  const hasPA =
    p.pa && Object.values(p.pa).some((arr: any) => Array.isArray(arr) && arr.length > 0);
  // 2. Détection de la couche EPBS
  const hasEPBS =
    p.epbs && Object.values(p.epbs).some((arr: any) => Array.isArray(arr) && arr.length > 0);
  const hasData =
    p.data && Object.values(p.data).some((arr: any) => Array.isArray(arr) && arr.length > 0);

  return (
    <div
      style={{ padding: '20px 40px', overflowY: 'auto', height: '100%', fontFamily: 'sans-serif' }}
    >
      <header style={{ marginBottom: 30, borderBottom: '1px solid #e5e7eb', paddingBottom: 20 }}>
        <h2 className="text-primary" style={{ margin: 0 }}>
          Explorateur de Modèle
        </h2>
        <div style={{ color: '#6b7280', marginTop: 5 }}>
          Projet : <strong>{p.meta?.name || p.name || 'Inconnu'}</strong>
          <span style={{ margin: '0 10px' }}>•</span>
          ID: <code style={{ fontSize: '0.85em' }}>{p.id || 'N/A'}</code>
        </div>
      </header>

      {/* OA */}
      {hasOA && (
        <div style={{ marginBottom: 30 }}>
          <div onClick={() => toggleLayer('oa')} style={styles.layerHeader('#f59e0b')}>
            <span>🌍 Analyse Opérationnelle (OA)</span>
            <span>{openLayers.oa ? '▼' : '▶'}</span>
          </div>
          {openLayers.oa && (
            <div style={styles.layerContent}>
              {renderElementList('Acteurs Opérationnels', p.oa.actors, '👤')}
              {renderElementList('Activités Opérationnelles', p.oa.activities, '⚙️')}
              {renderElementList('Entités', p.oa.entities, '🏢')}
            </div>
          )}
        </div>
      )}

      {/* SA */}
      {hasSA && (
        <div style={{ marginBottom: 30 }}>
          <div onClick={() => toggleLayer('sa')} style={styles.layerHeader('#10b981')}>
            <span>🔭 Analyse Système (SA)</span>
            <span>{openLayers.sa ? '▼' : '▶'}</span>
          </div>
          {openLayers.sa && (
            <div style={styles.layerContent}>
              {renderElementList('Acteurs Système', p.sa.actors, '👤')}
              {renderElementList('Capacités Système', p.sa.capabilities, '🎯')}
              {renderElementList('Fonctions Système', p.sa.functions, 'ƒ(x)')}
            </div>
          )}
        </div>
      )}

      {/* LA */}
      {hasLA && (
        <div style={{ marginBottom: 30 }}>
          <div onClick={() => toggleLayer('la')} style={styles.layerHeader('#3b82f6')}>
            <span>🧠 Architecture Logique (LA)</span>
            <span>{openLayers.la ? '▼' : '▶'}</span>
          </div>
          {openLayers.la && (
            <div style={styles.layerContent}>
              {renderElementList('Composants Logiques', p.la.components, '📦')}
              {renderElementList('Fonctions Logiques', p.la.functions, 'ƒ(x)')}
              {renderElementList('Acteurs Logiques', p.la.actors, '👤')}
            </div>
          )}
        </div>
      )}

      {/* PA */}
      {hasPA && (
        <div style={{ marginBottom: 30 }}>
          <div onClick={() => toggleLayer('pa')} style={styles.layerHeader('#8b5cf6')}>
            <span>⚙️ Architecture Physique (PA)</span>
            <span>{openLayers.pa ? '▼' : '▶'}</span>
          </div>
          {openLayers.pa && (
            <div style={styles.layerContent}>
              {renderElementList('Composants Physiques (Node)', p.pa.components, '🖥️')}
              {renderElementList('Acteurs Physiques', p.pa.actors, '👤')}
            </div>
          )}
        </div>
      )}

      {/* 3. COUCHE EPBS (Nouveau) */}
      {hasEPBS && (
        <div style={{ marginBottom: 30 }}>
          <div onClick={() => toggleLayer('epbs')} style={styles.layerHeader('#db2777')}>
            <span>📦 End Product Breakdown (EPBS)</span>
            <span>{openLayers.epbs ? '▼' : '▶'}</span>
          </div>
          {openLayers.epbs && (
            <div style={styles.layerContent}>
              {renderElementList('Articles de Configuration (CI)', p.epbs.configurationItems, '🍱')}
            </div>
          )}
        </div>
      )}

      {/* DATA */}
      <div style={{ marginBottom: 30 }}>
        <div onClick={() => toggleLayer('data')} style={styles.layerHeader('#6b7280')}>
          <span>📚 Dictionnaire de Données (Commun)</span>
          <span>{openLayers.data ? '▼' : '▶'}</span>
        </div>
        {openLayers.data && (
          <div style={styles.layerContent}>
            {!hasData ? (
              <div style={{ padding: 10, fontStyle: 'italic', color: '#9ca3af' }}>
                Aucune définition de donnée partagée.
              </div>
            ) : (
              <>
                {renderElementList('Classes', p.data?.classes, '🏷️')}
                {renderElementList('Types de Données', p.data?.dataTypes, '🔢')}
                {renderElementList("Items d'Échange", p.data?.exchangeItems, '🔄')}
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// Styles inchangés
const styles = {
  layerHeader: (color: string) => ({
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 16px',
    backgroundColor: 'white',
    borderLeft: `5px solid ${color}`,
    borderRadius: '6px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
    cursor: 'pointer',
    fontWeight: 'bold',
    fontSize: '1.1em',
    color: '#1f2937',
    marginBottom: 10,
    userSelect: 'none' as const,
  }),
  layerContent: {
    paddingLeft: 10,
    borderLeft: '1px dashed #e5e7eb',
    marginLeft: 20,
  },
};
