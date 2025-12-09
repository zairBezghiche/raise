// FICHIER : src/components/model-viewer/DataDictionary.tsx

import { useModelStore } from '@/store/model-store';
import { ArcadiaTypes } from '@/types/arcadia.types';
import type { ArcadiaElement } from '@/types/model.types';

export function DataDictionary() {
  const { project } = useModelStore();

  // 1. Si pas de projet, message simple
  if (!project) {
    return <div style={{ color: '#9ca3af', padding: 20 }}>Aucun modèle chargé.</div>;
  }

  // 2. SAFETY CHECK : Si la couche data est manquante (backend pas à jour ?)
  // On crée un objet vide par défaut pour éviter le crash "Cannot destructure property..."
  const dataLayer = project.data || { exchangeItems: [], classes: [], dataTypes: [] };
  const { exchangeItems, classes, dataTypes } = dataLayer;

  // 3. Pas de données ?
  if (
    (!exchangeItems || exchangeItems.length === 0) &&
    (!classes || classes.length === 0) &&
    (!dataTypes || dataTypes.length === 0)
  ) {
    return (
      <div style={{ padding: 20 }}>
        <h3 className="text-primary">Dictionnaire de Données</h3>
        <p className="text-gray">La couche Data est vide.</p>
      </div>
    );
  }

  return (
    <div style={{ padding: 20, overflowY: 'auto', height: '100%' }}>
      <h2 style={{ marginBottom: 20 }}>Dictionnaire de Données</h2>

      {/* On utilise l'opérateur ?. pour éviter le crash si ArcadiaTypes n'est pas chargé */}
      <Section
        title="📦 Exchange Items"
        elements={exchangeItems}
        type={ArcadiaTypes?.EXCHANGE_ITEM}
      />
      <Section title="📄 Classes de Données" elements={classes} type={ArcadiaTypes?.DATA_CLASS} />
      <Section title="🔢 Types de Données" elements={dataTypes} type={ArcadiaTypes?.DATA_TYPE} />
    </div>
  );
}

function Section({
  title,
  elements,
  type,
}: {
  title: string;
  elements: ArcadiaElement[];
  type?: string;
}) {
  if (!elements || elements.length === 0) return null;

  return (
    <div style={{ marginBottom: 30 }}>
      <h3 style={{ borderBottom: '1px solid #374151', paddingBottom: 8, marginBottom: 12 }}>
        {title} <span style={{ fontSize: '0.8em', color: '#6b7280' }}>({elements.length})</span>
      </h3>

      <div style={{ display: 'grid', gap: 10 }}>
        {elements.map((el) => (
          <div
            key={el.id}
            style={{
              backgroundColor: '#1f2937',
              padding: '10px 14px',
              borderRadius: 6,
              borderLeft: '4px solid #4f46e5',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <div>
              <div style={{ fontWeight: 600, color: '#f3f4f6' }}>
                {/* Gestion robuste du nom multilingue ou simple */}
                {typeof el.name === 'string' ? el.name : (el.name as any)?.fr || 'Sans nom'}
              </div>
              <div style={{ fontSize: '0.75em', color: '#9ca3af', fontFamily: 'monospace' }}>
                {/* Vérification du type seulement si 'type' est défini */}
                {type && el.type === type
                  ? 'Type Validé ✅'
                  : type
                  ? '⚠️ Type Inconnu'
                  : 'Type non vérifié'}
              </div>
            </div>
            <div style={{ fontSize: '0.75em', color: '#6b7280', fontFamily: 'monospace' }}>
              ID: {el.id ? el.id.slice(0, 8) : '????'}...
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
