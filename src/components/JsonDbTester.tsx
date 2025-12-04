import { useState, useEffect } from 'react';
import { collectionService } from '@/services/json-db/collection-service';
import { createQuery } from '@/services/json-db/query-service';
import { modelService } from '@/services/model-service';
import { useModelStore } from '@/store/model-store';
import { InputBar } from '@/components/ai-chat/InputBar';

export function JsonDbTester() {
  // --- État UI ---
  const [logs, setLogs] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState<'write' | 'search'>('write');

  // --- État Données ---
  const [targetCollection, setTargetCollection] = useState('actors'); // Par défaut 'actors' pour le test RAG
  const [items, setItems] = useState<any[]>([]);

  // --- État Insertion Manuelle (Pour le test RAG) ---
  const [jsonInput, setJsonInput] = useState<string>(
    JSON.stringify(
      {
        '@context': {
          oa: 'https://genaptitude.io/ontology/arcadia/oa#',
        },
        '@type': 'oa:OperationalActor',
        name: 'Opérateur de Drone',
        description: 'Personne chargée du pilotage manuel du drone via la station sol',
      },
      null,
      2,
    ),
  );

  // --- État Recherche ---
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [searchStats, setSearchStats] = useState<string>('');

  const setProject = useModelStore((s) => s.setProject);

  const addLog = (msg: string) =>
    setLogs((prev) => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev]);

  // Chargement initial
  useEffect(() => {
    refreshItems();
  }, [targetCollection]);

  // 1. Création de collection
  const handleCreateCollection = async () => {
    try {
      await collectionService.createCollection(targetCollection);
      addLog(`✅ Collection '${targetCollection}' créée (ou déjà existante).`);
      await refreshItems();
    } catch (e: any) {
      addLog(`❌ Erreur création: ${e}`);
    }
  };

  // 2. Insertion de document JSON brut
  const handleInsertJson = async () => {
    if (!targetCollection.trim()) {
      addLog('❌ Erreur: Le nom de la collection est vide !');
      return;
    }
    try {
      const doc = JSON.parse(jsonInput);
      const saved = await collectionService.insertDocument(targetCollection, doc);
      addLog(`✅ Document inséré dans '${targetCollection}' (ID: ${saved.id})`);
      await refreshItems();
    } catch (e: any) {
      addLog(`❌ Erreur insertion JSON: ${e}`);
    }
  };

  // 3. Rafraîchissement liste
  const refreshItems = async () => {
    try {
      const docs = await collectionService.listAll(targetCollection);
      if (Array.isArray(docs)) {
        setItems(docs.reverse());
      } else {
        setItems([]);
      }
    } catch (e: any) {
      // On ignore l'erreur si la collection n'existe pas encore
      setItems([]);
    }
  };

  // 4. Recherche
  const handleSearch = async (text: string) => {
    if (!text.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      const start = performance.now();
      const query = createQuery(targetCollection).where('name', 'Contains', text).limit(20).build();

      const results = await collectionService.queryDocuments(targetCollection, query);
      const duration = (performance.now() - start).toFixed(2);

      setSearchResults(results);
      setSearchStats(`${results.length} résultat(s) en ${duration}ms`);
      addLog(`🔍 Recherche "${text}" sur ${targetCollection}`);
    } catch (e: any) {
      addLog(`❌ Erreur recherche: ${e}`);
    }
  };

  // 5. Chargement Modèle Complet (Pour le RAG)
  const handleLoadModel = async () => {
    try {
      addLog('⏳ Chargement du modèle complet pour le RAG...');
      const model = await modelService.loadProjectModel('un2', '_system');
      setProject(model);
      addLog(`✅ Modèle chargé en mémoire ! (Prêt pour les questions IA)`);
    } catch (e: any) {
      addLog(`❌ Erreur chargement modèle: ${e}`);
    }
  };

  return (
    <div
      style={{
        padding: 20,
        background: '#111827',
        borderRadius: 8,
        border: '1px solid #374151',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          marginBottom: 15,
          alignItems: 'center',
        }}
      >
        <h3 style={{ color: 'white', margin: 0 }}>Admin DB</h3>
        <button
          onClick={handleLoadModel}
          style={{
            background: '#4f46e5',
            color: 'white',
            border: 'none',
            padding: '6px 12px',
            borderRadius: 4,
            cursor: 'pointer',
            fontSize: '0.9em',
          }}
        >
          🔄 Recharger Modèle (RAG)
        </button>
      </div>

      {/* Controls Collection */}
      <div style={{ background: '#1f2937', padding: 10, borderRadius: 8, marginBottom: 15 }}>
        <div style={{ display: 'flex', gap: 10, marginBottom: 10 }}>
          <input
            value={targetCollection}
            onChange={(e) => setTargetCollection(e.target.value)}
            placeholder="Nom collection (ex: actors)"
            style={{
              flex: 1,
              padding: 8,
              borderRadius: 4,
              border: '1px solid #374151',
              background: '#111827',
              color: 'white',
            }}
          />
          <button
            onClick={handleCreateCollection}
            style={{
              background: '#10b981',
              color: 'white',
              border: 'none',
              padding: '0 15px',
              borderRadius: 4,
              cursor: 'pointer',
            }}
          >
            Créer / Ouvrir
          </button>
        </div>

        <div style={{ display: 'flex', gap: 5 }}>
          <button
            onClick={() => setActiveTab('write')}
            style={{
              flex: 1,
              padding: 6,
              borderRadius: 4,
              border: 'none',
              cursor: 'pointer',
              background: activeTab === 'write' ? '#374151' : 'transparent',
              color: activeTab === 'write' ? 'white' : '#9ca3af',
            }}
          >
            Édition
          </button>
          <button
            onClick={() => setActiveTab('search')}
            style={{
              flex: 1,
              padding: 6,
              borderRadius: 4,
              border: 'none',
              cursor: 'pointer',
              background: activeTab === 'search' ? '#374151' : 'transparent',
              color: activeTab === 'search' ? 'white' : '#9ca3af',
            }}
          >
            Recherche
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        {activeTab === 'write' && (
          <div style={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 10 }}>
            {/* Zone d'insertion JSON */}
            <div style={{ display: 'flex', flexDirection: 'column', flex: 1 }}>
              <textarea
                value={jsonInput}
                onChange={(e) => setJsonInput(e.target.value)}
                style={{
                  flex: 1,
                  background: '#000',
                  color: '#a5f3fc',
                  fontFamily: 'monospace',
                  padding: 10,
                  borderRadius: 6,
                  border: '1px solid #374151',
                  resize: 'none',
                }}
              />
              <button
                onClick={handleInsertJson}
                style={{
                  marginTop: 8,
                  background: '#3b82f6',
                  color: 'white',
                  border: 'none',
                  padding: '8px',
                  borderRadius: 4,
                  cursor: 'pointer',
                }}
              >
                Insérer Document
              </button>
            </div>

            {/* Logs */}
            <div
              style={{
                height: '150px',
                background: '#000',
                padding: 10,
                overflowY: 'auto',
                color: '#4ade80',
                fontSize: '0.75em',
                borderRadius: 8,
                fontFamily: 'monospace',
              }}
            >
              {logs.map((l, i) => (
                <div key={i}>{l}</div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'search' && (
          <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <InputBar
              value={searchQuery}
              onChange={setSearchQuery}
              onSend={handleSearch}
              placeholder={`Rechercher dans ${targetCollection}...`}
            />
            <div
              style={{
                marginTop: 5,
                color: '#10b981',
                fontSize: '0.8em',
                textAlign: 'right',
                minHeight: '20px',
              }}
            >
              {searchStats}
            </div>

            <div style={{ flex: 1, overflowY: 'auto', marginTop: 10 }}>
              {(searchResults.length > 0 ? searchResults : items).map((item: any) => (
                <div
                  key={item.id}
                  style={{ background: '#1f2937', marginBottom: 8, padding: 10, borderRadius: 6 }}
                >
                  <div style={{ color: '#fff', fontWeight: 'bold' }}>{item.name || 'Sans nom'}</div>
                  <div style={{ fontSize: '0.8em', color: '#9ca3af', marginTop: 4 }}>
                    {item.description}
                  </div>
                  <div
                    style={{
                      fontSize: '0.7em',
                      color: '#6b7280',
                      marginTop: 4,
                      fontFamily: 'monospace',
                    }}
                  >
                    ID: {item.id}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
