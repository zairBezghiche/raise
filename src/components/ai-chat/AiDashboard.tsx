import { useState, useEffect } from 'react';
import { aiService, AiStatus, NlpResult } from '@/services/ai-service';

export default function AiDashboard() {
  const [activeTab, setActiveTab] = useState<'llm' | 'nlp' | 'context' | 'agents'>('llm');
  const [status, setStatus] = useState<AiStatus | null>(null);
  const [nlpInput, setNlpInput] = useState(
    "L'ingénierie système nécessite une analyse rigoureuse.",
  );
  const [nlpResult, setNlpResult] = useState<NlpResult | null>(null);

  useEffect(() => {
    let isMounted = true;
    const fetchStatus = async () => {
      try {
        const s = await aiService.getSystemStatus();
        if (isMounted) setStatus(s);
      } catch (e: unknown) {
        console.error(e);
      }
    };
    fetchStatus();
    return () => {
      isMounted = false;
    };
  }, []);

  const runNlpTest = async () => {
    try {
      const res = await aiService.testNlp(nlpInput);
      setNlpResult(res);
    } catch (e: unknown) {
      console.error(e);
    }
  };

  // Styles (Je garde le style existant mais je le type implicitement)
  const styles = {
    // ... (copier les styles précédents, ils n'ont pas d'erreurs de type)
    container: {
      height: '100%',
      display: 'flex',
      flexDirection: 'column' as const,
      background: '#f3f4f6',
      fontFamily: 'Inter, sans-serif',
    },
    tabs: {
      display: 'flex',
      background: '#fff',
      borderBottom: '1px solid #e5e7eb',
      padding: '0 20px',
    },
    tab: (active: boolean) => ({
      padding: '12px 20px',
      cursor: 'pointer',
      fontWeight: 600,
      fontSize: '0.9rem',
      color: active ? '#4f46e5' : '#6b7280',
      borderBottom: active ? '2px solid #4f46e5' : '2px solid transparent',
    }),
    content: { flex: 1, overflow: 'auto', padding: '20px' },
    card: {
      background: 'white',
      padding: '25px',
      borderRadius: '12px',
      boxShadow: '0 1px 3px rgba(0,0,0,0.05)',
      marginBottom: '20px',
    },
    grid: {
      display: 'grid',
      gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
      gap: '20px',
    },
    statBox: {
      background: '#f9fafb',
      padding: '15px',
      borderRadius: '8px',
      border: '1px solid #e5e7eb',
    },
    label: {
      color: '#6b7280',
      fontSize: '0.85em',
      textTransform: 'uppercase' as const,
      letterSpacing: '0.05em',
    },
    value: { fontSize: '1.5em', fontWeight: 'bold', color: '#111827', marginTop: '5px' },
  };

  return (
    <div style={styles.container}>
      <div style={{ padding: '20px 20px 0', background: 'white' }}>
        <h2 style={{ margin: 0, color: '#111827' }}>AI Studio</h2>
        <p style={{ color: '#6b7280', fontSize: '0.9em', margin: '5px 0 15px' }}>
          Console de diagnostic des modules d'Intelligence Artificielle.
        </p>
      </div>

      <div style={styles.tabs}>
        <div style={styles.tab(activeTab === 'llm')} onClick={() => setActiveTab('llm')}>
          🔌 LLM Kernel
        </div>
        <div style={styles.tab(activeTab === 'nlp')} onClick={() => setActiveTab('nlp')}>
          ✂️ NLP Engine
        </div>
        <div style={styles.tab(activeTab === 'context')} onClick={() => setActiveTab('context')}>
          🗂️ RAG Context
        </div>
        <div style={styles.tab(activeTab === 'agents')} onClick={() => setActiveTab('agents')}>
          🤖 Agents
        </div>
      </div>

      <div style={styles.content}>
        {/* LLM */}
        {activeTab === 'llm' && (
          <div style={styles.card}>
            <h3>État du Noyau LLM</h3>
            <div style={styles.grid}>
              <div style={styles.statBox}>
                <div style={styles.label}>Statut</div>
                <div
                  style={{ ...styles.value, color: status?.llm_connected ? '#10b981' : '#ef4444' }}
                >
                  {status?.llm_connected ? '🟢 ONLINE' : '🔴 OFFLINE'}
                </div>
              </div>
              <div style={styles.statBox}>
                <div style={styles.label}>Modèle Chargé</div>
                <div style={styles.value}>{status?.llm_model || 'Aucun'}</div>
              </div>
              <div style={styles.statBox}>
                <div style={styles.label}>Provider</div>
                <div style={styles.value}>OpenAI / Local</div>
              </div>
            </div>
          </div>
        )}

        {/* NLP */}
        {activeTab === 'nlp' && (
          <div style={styles.card}>
            <h3>Moteur de Traitement du Langage</h3>
            <div style={{ marginBottom: 15 }}>
              <label style={styles.label}>Test Tokenizer</label>
              <div style={{ display: 'flex', gap: 10, marginTop: 5 }}>
                <input
                  value={nlpInput}
                  onChange={(e) => setNlpInput(e.target.value)}
                  style={{
                    flex: 1,
                    padding: '10px',
                    border: '1px solid #d1d5db',
                    borderRadius: '6px',
                  }}
                />
                <button
                  onClick={runNlpTest}
                  style={{
                    padding: '10px 20px',
                    background: '#4f46e5',
                    color: 'white',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                  }}
                >
                  Analyser
                </button>
              </div>
            </div>
            {nlpResult && (
              <div style={{ background: '#f3f4f6', padding: 15, borderRadius: 8 }}>
                <div style={{ marginBottom: 10, fontWeight: 'bold' }}>
                  {nlpResult.token_count} Tokens détectés :
                </div>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
                  {/* Correction : Typage explicite dans le map */}
                  {nlpResult.tokens.map((t: string, i: number) => (
                    <span
                      key={i}
                      style={{
                        background: 'white',
                        padding: '4px 8px',
                        borderRadius: 4,
                        border: '1px solid #e5e7eb',
                        fontFamily: 'monospace',
                      }}
                    >
                      {t}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* CONTEXT */}
        {activeTab === 'context' && (
          <div style={styles.card}>
            <h3>Mémoire Vectorielle (RAG)</h3>
            <div style={{ display: 'flex', alignItems: 'center', gap: 30 }}>
              <div
                style={{
                  width: 120,
                  height: 120,
                  borderRadius: '50%',
                  background: '#e0e7ff',
                  color: '#4338ca',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  fontSize: '2.5rem',
                  fontWeight: 'bold',
                }}
              >
                {status?.context_documents || 0}
              </div>
              <div>
                <h4 style={{ margin: '0 0 5px' }}>Documents Vectorisés</h4>
                <p style={{ margin: 0, color: '#6b7280' }}>
                  Le contexte est injecté dynamiquement dans les prompts LLM.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* AGENTS */}
        {activeTab === 'agents' && (
          <div style={styles.card}>
            <h3>Orchestration Multi-Agents</h3>
            <div style={{ display: 'grid', gap: 10 }}>
              {status?.active_agents.map((agent: string, i: number) => (
                <div
                  key={i}
                  style={{
                    padding: 15,
                    border: '1px solid #e5e7eb',
                    borderRadius: 8,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 15 }}>
                    <span
                      style={{
                        background: '#d1fae5',
                        padding: 8,
                        borderRadius: 6,
                        fontSize: '1.2em',
                      }}
                    >
                      🤖
                    </span>
                    <span style={{ fontWeight: 600 }}>{agent}</span>
                  </div>
                  <span
                    style={{
                      fontSize: '0.8em',
                      background: '#ecfdf5',
                      color: '#047857',
                      padding: '2px 8px',
                      borderRadius: 10,
                    }}
                  >
                    ● Actif
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
