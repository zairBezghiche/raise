import { useState, useEffect, useCallback } from 'react';
import { collectionService } from '@/services/json-db/collection-service';
import { createQuery } from '@/services/json-db/query-service';
import { modelService } from '@/services/model-service';
import { useModelStore } from '@/store/model-store';
import { InputBar } from '@/components/ai-chat/InputBar';

export function JsonDbTester() {
  // --- État UI ---
  const [logs, setLogs] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState<'write' | 'search' | 'admin'>('admin');

  // --- État Données ---
  const [targetCollection, setTargetCollection] = useState('actors');
  const [items, setItems] = useState<any[]>([]);

  // --- État Insertion Manuelle ---
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

  // --- HELPER : Récupération des données (Logic Only) ---
  const fetchCollectionData = useCallback(async (colName: string) => {
    try {
      const docs = await collectionService.listAll(colName);
      return Array.isArray(docs) ? docs.reverse() : [];
    } catch (e) {
      return [];
    }
  }, []);

  // --- 1. CHARGEMENT AUTOMATIQUE (Sécurisé) ---
  useEffect(() => {
    let isMounted = true;

    const load = async () => {
      const data = await fetchCollectionData(targetCollection);
      if (isMounted) {
        setItems(data);
      }
    };

    load();

    return () => {
      isMounted = false;
    };
  }, [targetCollection, fetchCollectionData]);

  // --- 2. ACTION MANUELLE (Pour les boutons) ---
  const handleRefreshManual = async () => {
    const data = await fetchCollectionData(targetCollection);
    setItems(data);
  };

  // --- ACTIONS ADMIN ---
  const handleInitDb = async () => {
    try {
      await collectionService.createDb();
      addLog('✅ Base de données initialisée (un2/_system).');
    } catch (e: any) {
      addLog(`❌ Erreur Init DB: ${e}`);
    }
  };

  const handleDropDb = async () => {
    if (!confirm('Êtes-vous sûr de vouloir tout supprimer ?')) return;
    try {
      await collectionService.dropDb();
      addLog('🗑️ Base de données supprimée.');
      setItems([]);
    } catch (e: any) {
      addLog(`❌ Erreur Drop DB: ${e}`);
    }
  };

  const handleCreateIndex = async () => {
    try {
      await collectionService.createIndex(targetCollection, 'name', 'hash');
      addLog(`✅ Index Hash créé sur ${targetCollection}.name`);
    } catch (e: any) {
      addLog(`❌ Erreur Index: ${e}`);
    }
  };

  // --- ACTIONS COLLECTION ---
  const handleCreateCollection = async () => {
    try {
      await collectionService.createCollection(targetCollection);
      addLog(`✅ Collection '${targetCollection}' prête.`);
      await handleRefreshManual();
    } catch (e: any) {
      addLog(`❌ Erreur création collection: ${e}`);
    }
  };

  const handleInsertJson = async () => {
    if (!targetCollection.trim()) {
      addLog('❌ Erreur: Nom de collection vide !');
      return;
    }
    try {
      const doc = JSON.parse(jsonInput);
      const saved = await collectionService.insertDocument(targetCollection, doc);
      addLog(`✅ Document inséré (ID: ${saved.id})`);
      await handleRefreshManual();
    } catch (e: any) {
      addLog(`❌ Erreur insertion: ${e}`);
    }
  };

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
      addLog(`🔍 Recherche "${text}" terminée.`);
    } catch (e: any) {
      addLog(`❌ Erreur recherche: ${e}`);
    }
  };

  const handleLoadModel = async () => {
    try {
      addLog('⏳ Chargement du modèle RAG...');
      const model = await modelService.loadProjectModel('un2', '_system');
      setProject(model);
      addLog(`✅ Modèle chargé !`);
    } catch (e: any) {
      addLog(`❌ Erreur modèle: ${e}`);
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
      {/* Header & Tabs */}
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 15 }}>
        <h3 style={{ color: 'white', margin: 0 }}>JSON-DB Explorer</h3>
        <div style={{ display: 'flex', gap: 5 }}>
          {['admin', 'write', 'search'].map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab as any)}
              style={{
                padding: '6px 12px',
                borderRadius: 4,
                border: 'none',
                cursor: 'pointer',
                background: activeTab === tab ? '#4f46e5' : '#374151',
                color: 'white',
                textTransform: 'capitalize',
              }}
            >
              {tab}
            </button>
          ))}
        </div>
      </div>

      {/* Barre de collection (Toujours visible) */}
      <div style={{ display: 'flex', gap: 10, marginBottom: 15 }}>
        <input
          value={targetCollection}
          onChange={(e) => setTargetCollection(e.target.value)}
          placeholder="Collection (ex: actors)"
          style={{
            flex: 1,
            padding: 8,
            borderRadius: 4,
            border: '1px solid #374151',
            background: '#1f2937',
            color: 'white',
          }}
        />
        <button
          onClick={handleCreateCollection}
          style={{
            background: '#10b981',
            color: 'white',
            border: 'none',
            borderRadius: 4,
            padding: '0 15px',
            cursor: 'pointer',
          }}
        >
          Ouvrir / Créer
        </button>
      </div>

      {/* CONTENU PRINCIPAL */}
      <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        {/* --- ONGLET ADMIN --- */}
        {activeTab === 'admin' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 15 }}>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
              <button
                onClick={handleInitDb}
                style={{
                  padding: 15,
                  background: '#059669',
                  color: 'white',
                  border: 'none',
                  borderRadius: 6,
                  cursor: 'pointer',
                }}
              >
                🏗️ Initialiser DB
              </button>
              <button
                onClick={handleDropDb}
                style={{
                  padding: 15,
                  background: '#dc2626',
                  color: 'white',
                  border: 'none',
                  borderRadius: 6,
                  cursor: 'pointer',
                }}
              >
                💥 Supprimer DB
              </button>
              <button
                onClick={handleCreateIndex}
                style={{
                  padding: 15,
                  background: '#d97706',
                  color: 'white',
                  border: 'none',
                  borderRadius: 6,
                  cursor: 'pointer',
                }}
              >
                ⚡ Indexer 'name'
              </button>
              <button
                onClick={handleLoadModel}
                style={{
                  padding: 15,
                  background: '#2563eb',
                  color: 'white',
                  border: 'none',
                  borderRadius: 6,
                  cursor: 'pointer',
                }}
              >
                🧠 Charger Modèle (RAG)
              </button>
            </div>

            {/* Logs Console */}
            <div
              style={{
                flex: 1,
                background: '#000',
                padding: 10,
                overflowY: 'auto',
                color: '#4ade80',
                fontSize: '0.8em',
                fontFamily: 'monospace',
                borderRadius: 6,
                border: '1px solid #374151',
              }}
            >
              {logs.length === 0 && (
                <span style={{ color: '#6b7280' }}>En attente d'actions...</span>
              )}
              {logs.map((l, i) => (
                <div key={i}>{l}</div>
              ))}
            </div>
          </div>
        )}

        {/* --- ONGLET WRITE --- */}
        {activeTab === 'write' && (
          <div style={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 10 }}>
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
                background: '#3b82f6',
                color: 'white',
                border: 'none',
                padding: 12,
                borderRadius: 6,
                cursor: 'pointer',
                fontWeight: 'bold',
              }}
            >
              💾 Insérer Document
            </button>
          </div>
        )}

        {/* --- ONGLET SEARCH --- */}
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
                  style={{
                    background: '#1f2937',
                    marginBottom: 8,
                    padding: 10,
                    borderRadius: 6,
                    borderLeft: '3px solid #6366f1',
                  }}
                >
                  <div style={{ color: '#fff', fontWeight: 'bold' }}>
                    {item.name || item.title || 'Sans nom'}
                  </div>
                  <div style={{ fontSize: '0.8em', color: '#9ca3af', marginTop: 4 }}>
                    {item.description}
                  </div>
                  <div
                    style={{
                      fontSize: '0.7em',
                      color: '#6b7280',
                      marginTop: 6,
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
