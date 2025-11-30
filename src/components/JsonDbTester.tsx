import { useState, useRef, useEffect } from 'react'
import { collectionService } from '@/services/json-db/collection-service'
import { createTransaction, TransactionService } from '@/services/json-db/transaction-service'
import { createQuery } from '@/services/json-db/query-service'
import { Button } from '@/components/shared/Button'
import { InputBar } from '@/components/ai-chat/InputBar'
import type { OperationRequest } from '@/types/json-db.types'
import { invoke } from '@tauri-apps/api/core';

// Styles pour les badges d'opération
const OP_STYLES = {
  insert: { color: '#4ade80', label: 'INSERT' }, // Vert
  update: { color: '#60a5fa', label: 'UPDATE' }, // Bleu
  delete: { color: '#f87171', label: 'DELETE' }  // Rouge
};

const COLLECTION_NAME = 'smoke_test_transactions';

export function JsonDbTester() {
  const [activeTab, setActiveTab] = useState<'write' | 'search'>('write');
  const [logs, setLogs] = useState<string[]>([])
  
  // État pour la lecture
  const [items, setItems] = useState<any[]>([])
  
  // État pour la recherche
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<any[]>([])
  const [searchStats, setSearchStats] = useState<string>('')
  
  // État pour les transactions
  const [pendingOps, setPendingOps] = useState<OperationRequest[]>([])
  const txRef = useRef<TransactionService>(createTransaction())

  const addLog = (msg: string) => setLogs((prev) => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev])

  // Chargement initial
  useEffect(() => {
    initCollection();
  }, []);

  const initCollection = async () => {
    try {
      // Création de la collection avec un schéma générique si elle n'existe pas
      await collectionService.createCollection(COLLECTION_NAME).catch(() => {});
      await refreshItems();
    } catch (e: any) {
      addLog(`⚠️ Erreur init: ${e}`);
    }
  }

  // --- MODE LECTURE / ÉCRITURE ---

  const refreshItems = async () => {
    try {
      const docs = await collectionService.listAll(COLLECTION_NAME);
      setItems(docs.reverse()); // Plus récents en haut
    } catch (e: any) {
      addLog(`⚠️ Erreur lecture: ${e}`);
    }
  }

  // --- MODE RECHERCHE (Query Engine) ---

  const handleSearch = async (text: string) => {
    if (!text.trim()) {
      setSearchResults([]);
      setSearchStats('');
      return;
    }

    try {
      const start = performance.now();
      
      // Construction de la requête
      // On cherche dans 'name' OU 'status' OU 'id'
      const query = createQuery(COLLECTION_NAME)
        .where('name', 'contains', text)
        .or({ op: 'contains', field: 'status', value: text })
        .or({ op: 'eq', field: 'id', value: text }) // Recherche exacte par ID
        .orderBy('updatedAt', 'desc')
        .limit(20)
        .build();

      // Exécution via le service qui appelle le backend Rust (jsondb_query_collection)
      const results = await collectionService.queryDocuments(COLLECTION_NAME, query);
      
      const duration = (performance.now() - start).toFixed(2);
      setSearchResults(results);
      setSearchStats(`${results.length} résultat(s) en ${duration}ms`);
      addLog(`🔍 Recherche "${text}" : ${results.length} hits (${duration}ms)`);
      
    } catch (e: any) {
      addLog(`❌ Erreur recherche: ${e}`);
    }
  }

  // --- ACTIONS TRANSACTIONNELLES (ACID) ---

  const stageInsert = () => {
    const docName = `Document ${items.length + pendingOps.length + 1}`;
    
    txRef.current.add(COLLECTION_NAME, { 
      name: docName, 
      status: 'draft',
      // Ajout d'un texte aléatoire pour tester l'indexation textuelle
      description: Math.random() > 0.5 ? "Projet critique confidentiel" : "Note publique archivée",
      updatedAt: new Date().toISOString()
    });
    
    addLog(`📝 Staged: INSERT "${docName}"`);
    setPendingOps(txRef.current.getPendingOperations());
  }

  const stageUpdate = (doc: any) => {
    const newDoc = { 
      ...doc, 
      name: `${doc.name} (edited)`, 
      status: 'published',
      updatedAt: new Date().toISOString()
    };

    txRef.current.update(COLLECTION_NAME, newDoc);
    addLog(`📝 Staged: UPDATE "${doc.id}"`);
    setPendingOps(txRef.current.getPendingOperations());
  }

  const stageDelete = (id: string) => {
    txRef.current.delete(COLLECTION_NAME, id);
    addLog(`📝 Staged: DELETE "${id}"`);
    setPendingOps(txRef.current.getPendingOperations());
  }

  const handleCommit = async () => {
    if (pendingOps.length === 0) return;
    
    try {
      addLog(`🚀 Committing ${pendingOps.length} operations...`);
      await txRef.current.commit();
      addLog(`✅ Transaction Committed (ACID) !`);
      
      setPendingOps([]);
      await refreshItems();
      // Si on est en mode recherche, on relance la recherche pour voir les changements
      if (activeTab === 'search' && searchQuery) {
        handleSearch(searchQuery);
      }
      
    } catch (e: any) {
      addLog(`❌ Transaction Failed: ${e}`);
    }
  }

  const handleRollback = () => {
    txRef.current.rollback();
    setPendingOps([]);
    addLog(`↩️ Rollback effectué`);
  }

  const testLoad = async () => {
    try {
      addLog("⏳ Chargement du modèle complet en mémoire (Rust)...");
      const start = performance.now();
      
      // Appel de la commande définie dans model_commands.rs
      // Le type de retour est 'any' pour l'instant, mais correspond à ProjectModel
      const model: any = await invoke('load_project_model', { 
        space: 'un2', 
        db: '_system' 
      });
      
      const duration = (performance.now() - start).toFixed(2);
      
      // Extraction de quelques stats pour vérifier le contenu
      const oaActors = model.oa?.actors?.length || 0;
      const saFunctions = model.sa?.functions?.length || 0;
      const paComponents = model.pa?.components?.length || 0;
      
      addLog(`✅ Modèle chargé en ${duration}ms !`);
      addLog(`📊 Stats: ${oaActors} Acteurs OA / ${saFunctions} Fonctions SA / ${paComponents} Composants PA`);
      
      // Log complet dans la console développeur du navigateur (F12) pour inspection
      console.log("📦 Modèle Complet :", model);

    } catch (e: any) {
      addLog(`❌ Erreur de chargement du modèle : ${e}`);
      console.error(e);
    }
  }  
  // --- RENDERERS ---

  const renderDocItem = (item: any, showActions: boolean) => (
    <div key={item.id} style={{ background: '#1f2937', padding: 12, borderRadius: 6, border: '1px solid #374151', display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
      <div style={{ overflow: 'hidden' }}>
        <div style={{ color: '#f3f4f6', fontWeight: 500 }}>{item.name || 'Sans nom'}</div>
        <div style={{ color: '#9ca3af', fontSize: '0.8em', fontFamily: 'monospace' }}>ID: {item.id}</div>
        <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
          <span style={{ fontSize: '0.75em', background: '#374151', padding: '2px 6px', borderRadius: 4, color: '#d1d5db' }}>
            {item.status || 'N/A'}
          </span>
          {item.description && (
             <span style={{ fontSize: '0.75em', color: '#6b7280', fontStyle: 'italic' }}>
               {item.description}
             </span>
          )}
        </div>
      </div>
      
      {showActions && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <button 
            onClick={() => stageUpdate(item)}
            style={{ border: '1px solid #3b82f6', background: 'transparent', color: '#60a5fa', padding: '4px 8px', borderRadius: 4, cursor: 'pointer', fontSize: '0.8em' }}
          >
            Edit
          </button>
          <button 
            onClick={() => stageDelete(item.id)}
            style={{ border: '1px solid #ef4444', background: 'transparent', color: '#f87171', padding: '4px 8px', borderRadius: 4, cursor: 'pointer', fontSize: '0.8em' }}
          >
            Del
          </button>
        </div>
      )}
    </div>
  );

  return (
    <div style={{ padding: 20, background: '#111827', borderRadius: 8, border: '1px solid #374151', marginTop: 20, height: '650px', display: 'flex', flexDirection: 'column' }}>
      
      {/* HEADER */}
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'center', marginBottom: 16}}>
        <div>
            <h3 style={{ color: '#fff', margin: 0 }}>⚛️ Moteur JSON-DB</h3>
            <div style={{fontSize: '0.8em', color: '#6b7280'}}>ACID Transactions & Search Engine</div>
        </div>
        <div style={{display: 'flex', gap: 8}}> {/* Ajout d'un gap pour espacer les groupes */}
            
            {/* NOUVEAU BOUTON DE TEST DE CHARGEMENT */}
            <button 
                onClick={testLoad}
                style={{
                    background: '#4f46e5', // Indigo
                    color: '#fff',
                    border: 'none', padding: '6px 12px', borderRadius: 6, cursor: 'pointer', fontSize: '0.9em', fontWeight: 500,
                    display: 'flex', alignItems: 'center', gap: 4
                }}
            >
                📂 Charger Modèle
            </button>

            <div style={{display: 'flex', background: '#1f2937', padding: 4, borderRadius: 8}}>
                <button 
                    onClick={() => setActiveTab('write')}
                    style={{
                        background: activeTab === 'write' ? '#374151' : 'transparent',
                        color: activeTab === 'write' ? '#fff' : '#9ca3af',
                        border: 'none', padding: '6px 12px', borderRadius: 6, cursor: 'pointer', fontSize: '0.9em', fontWeight: 500
                    }}
                >
                    ✍️ Transactions
                </button>
                <button 
                    onClick={() => setActiveTab('search')}
                    style={{
                        background: activeTab === 'search' ? '#374151' : 'transparent',
                        color: activeTab === 'search' ? '#fff' : '#9ca3af',
                        border: 'none', padding: '6px 12px', borderRadius: 6, cursor: 'pointer', fontSize: '0.9em', fontWeight: 500
                    }}
                >
                    🔍 Recherche
                </button>
            </div>
        </div>
                
        <div style={{display: 'flex', background: '#1f2937', padding: 4, borderRadius: 8}}>
            <button 
                onClick={() => setActiveTab('write')}
                style={{
                    background: activeTab === 'write' ? '#374151' : 'transparent',
                    color: activeTab === 'write' ? '#fff' : '#9ca3af',
                    border: 'none', padding: '6px 12px', borderRadius: 6, cursor: 'pointer', fontSize: '0.9em', fontWeight: 500
                }}
            >
                ✍️ Transactions
            </button>
            <button 
                onClick={() => setActiveTab('search')}
                style={{
                    background: activeTab === 'search' ? '#374151' : 'transparent',
                    color: activeTab === 'search' ? '#fff' : '#9ca3af',
                    border: 'none', padding: '6px 12px', borderRadius: 6, cursor: 'pointer', fontSize: '0.9em', fontWeight: 500
                }}
            >
                🔍 Recherche
            </button>
        </div>
      </div>
      
      <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: 20, flex: 1, overflow: 'hidden' }}>
        
        {/* COLONNE GAUCHE : Staging Area (Toujours visible) */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10, borderRight: '1px solid #374151', paddingRight: 20 }}>
          <div style={{ background: '#1f2937', padding: 12, borderRadius: 8 }}>
            <Button onClick={stageInsert} style={{width: '100%', marginBottom: 10}}>
              + Nouvel Élément
            </Button>
            <div style={{ display: 'flex', gap: 8 }}>
              <Button 
                onClick={handleCommit} 
                disabled={pendingOps.length === 0}
                style={{ flex: 1, backgroundColor: pendingOps.length > 0 ? '#10b981' : '#374151' }}
              >
                Commit
              </Button>
              <Button 
                onClick={handleRollback} 
                disabled={pendingOps.length === 0}
                style={{ flex: 1, background: 'transparent', border: '1px solid #ef4444', color: '#ef4444', opacity: pendingOps.length === 0 ? 0.5 : 1 }}
              >
                Rollback
              </Button>
            </div>
          </div>

          <div style={{ flex: 1, background: '#0f172a', borderRadius: 8, padding: 10, overflowY: 'auto', border: '1px solid #1e293b' }}>
            <h4 style={{ color: '#94a3b8', marginTop: 0, fontSize: '0.8em', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Modifications ({pendingOps.length})
            </h4>
            {pendingOps.length === 0 ? (
              <div style={{ color: '#475569', textAlign: 'center', padding: 20, fontSize: '0.85em', fontStyle: 'italic' }}>
                Zone de transit vide.
              </div>
            ) : (
              <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
                {pendingOps.map((op, i) => (
                    <li key={i} style={{ fontSize: '0.85em', background: '#1e293b', marginBottom: 6, padding: 8, borderRadius: 4, borderLeft: `3px solid ${OP_STYLES[op.type].color}` }}>
                      <div style={{display: 'flex', justifyContent: 'space-between'}}>
                        <span style={{ color: OP_STYLES[op.type].color, fontWeight: 'bold' }}>{OP_STYLES[op.type].label}</span>
                      </div>
                      <div style={{ color: '#e2e8f0', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', marginTop: 2 }}>
                        {op.type === 'delete' ? op.id : (op.doc.name || 'Doc')}
                      </div>
                    </li>
                ))}
              </ul>
            )}
          </div>
          
          {/* Logs Console */}
          <div style={{ height: 150, background: '#000', padding: 8, borderRadius: 8, overflowY: 'auto', fontSize: '0.7em', fontFamily: 'monospace', color: '#4ade80' }}>
            {logs.map((l, i) => <div key={i}>{l}</div>)}
          </div>
        </div>

        {/* COLONNE DROITE : Contenu Principal */}
        <div style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          
          {/* MODE ÉCRITURE */}
          {activeTab === 'write' && (
            <>
              <div style={{ paddingBottom: 12, borderBottom: '1px solid #374151', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h4 style={{ color: '#e5e7eb', margin: 0 }}>Collection Complète ({items.length})</h4>
                <button onClick={refreshItems} style={{background:'none', border:'none', color:'#60a5fa', cursor:'pointer', fontSize: '0.9em'}}>
                  ↻ Actualiser
                </button>
              </div>
              <div style={{ flex: 1, overflowY: 'auto', paddingTop: 12 }}>
                {items.length === 0 ? (
                  <div style={{ textAlign: 'center', color: '#6b7280', marginTop: 40 }}>Collection vide.</div>
                ) : (
                  items.map(item => renderDocItem(item, true))
                )}
              </div>
            </>
          )}

          {/* MODE RECHERCHE */}
          {activeTab === 'search' && (
            <>
               <div style={{ paddingBottom: 12 }}>
                  <InputBar 
                    value={searchQuery} 
                    onChange={(val) => { setSearchQuery(val); handleSearch(val); }} 
                    onSend={() => {}}
                    placeholder="Rechercher (ex: 'critique', 'draft', 'uuid')..."
                  />
                  {searchStats && (
                      <div style={{ fontSize: '0.8em', color: '#10b981', marginTop: 8, textAlign: 'right' }}>
                          {searchStats}
                      </div>
                  )}
               </div>
               <div style={{ flex: 1, overflowY: 'auto', borderTop: '1px solid #374151', paddingTop: 12 }}>
                 {searchResults.length === 0 ? (
                     <div style={{ textAlign: 'center', color: '#6b7280', marginTop: 40 }}>
                         {searchQuery ? "Aucun résultat trouvé." : "Tapez pour rechercher dans les index."}
                     </div>
                 ) : (
                     searchResults.map(item => renderDocItem(item, false))
                 )}
               </div>
            </>
          )}

        </div>
      </div>
    </div>
  )
}