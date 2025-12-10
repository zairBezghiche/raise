import { useState } from 'react';
import { geneticsService, GeneticsParams, OptimizationResult } from '@/services/geneticsService';
import { useModelStore } from '@/store/model-store';

export default function GeneticsDashboard() {
  const currentProject = useModelStore((state) => state.project);

  // Paramètres de simulation
  const [params, setParams] = useState<GeneticsParams>({
    population_size: 100,
    generations: 50,
    mutation_rate: 0.5,
  });

  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<OptimizationResult | null>(null);

  const handleRun = async () => {
    setLoading(true);
    setResult(null);
    try {
      // On passe le modèle actuel (même s'il est mocké côté Rust pour l'instant)
      const res = await geneticsService.runOptimization(params, currentProject || {});
      setResult(res);
    } catch (e) {
      alert("Erreur lors de l'optimisation : " + e);
    } finally {
      setLoading(false);
    }
  };

  // Styles
  const styles = {
    container: {
      padding: '20px',
      color: '#e5e7eb',
      fontFamily: 'Inter, sans-serif',
      height: '100%',
      overflowY: 'auto' as const,
    },
    grid: { display: 'grid', gridTemplateColumns: '300px 1fr', gap: '20px' },
    panel: {
      background: '#1f2937',
      padding: '20px',
      borderRadius: '8px',
      border: '1px solid #374151',
    },
    label: { display: 'block', marginBottom: '8px', fontSize: '0.9em', color: '#9ca3af' },
    inputGroup: { marginBottom: '20px' },
    range: { width: '100%', accentColor: 'var(--color-primary)' },
    valueDisplay: { float: 'right' as const, color: 'var(--color-primary)', fontWeight: 'bold' },
    btn: {
      width: '100%',
      padding: '12px',
      background: 'linear-gradient(90deg, #ec4899, #8b5cf6)',
      color: 'white',
      border: 'none',
      borderRadius: '6px',
      fontWeight: 'bold',
      cursor: 'pointer',
      opacity: loading ? 0.7 : 1,
    },
    chartBar: (val: number) => ({
      height: `${val}%`,
      width: '20px',
      background: '#8b5cf6',
      borderRadius: '2px 2px 0 0',
      transition: 'height 0.5s ease',
    }),
    chartContainer: {
      display: 'flex',
      alignItems: 'flex-end',
      gap: '5px',
      height: '200px',
      borderBottom: '2px solid #4b5563',
      paddingBottom: '5px',
      marginTop: '20px',
    },
  };

  return (
    <div style={styles.container}>
      <header style={{ marginBottom: '20px' }}>
        <h2 style={{ margin: 0, color: '#f472b6' }}>Optimisation Génétique</h2>
        <p style={{ color: '#9ca3af' }}>
          Exploration de l'espace de conception par sélection naturelle simulée.
        </p>
      </header>

      <div style={styles.grid}>
        {/* --- PANNEAU DE CONTRÔLE --- */}
        <div style={styles.panel}>
          <h3 style={{ marginTop: 0 }}>Paramètres</h3>

          <div style={styles.inputGroup}>
            <label style={styles.label}>
              Taille Population <span style={styles.valueDisplay}>{params.population_size}</span>
            </label>
            <input
              type="range"
              min="10"
              max="1000"
              step="10"
              style={styles.range}
              value={params.population_size}
              onChange={(e) => setParams({ ...params, population_size: +e.target.value })}
            />
          </div>

          <div style={styles.inputGroup}>
            <label style={styles.label}>
              Générations <span style={styles.valueDisplay}>{params.generations}</span>
            </label>
            <input
              type="range"
              min="10"
              max="500"
              step="10"
              style={styles.range}
              value={params.generations}
              onChange={(e) => setParams({ ...params, generations: +e.target.value })}
            />
          </div>

          <div style={styles.inputGroup}>
            <label style={styles.label}>
              Taux Mutation <span style={styles.valueDisplay}>{params.mutation_rate}</span>
            </label>
            <input
              type="range"
              min="0.1"
              max="1.0"
              step="0.1"
              style={styles.range}
              value={params.mutation_rate}
              onChange={(e) => setParams({ ...params, mutation_rate: +e.target.value })}
            />
          </div>

          <button style={styles.btn} onClick={handleRun} disabled={loading}>
            {loading ? '🧬 Évolution en cours...' : "Lancer l'Optimisation"}
          </button>
        </div>

        {/* --- PANNEAU RÉSULTATS --- */}
        <div style={styles.panel}>
          <h3 style={{ marginTop: 0 }}>Convergence</h3>

          {!result && !loading && (
            <div
              style={{
                color: '#6b7280',
                fontStyle: 'italic',
                marginTop: '50px',
                textAlign: 'center',
              }}
            >
              Configurez les paramètres et lancez l'algorithme pour voir les résultats.
            </div>
          )}

          {loading && (
            <div style={{ textAlign: 'center', marginTop: '50px' }}>
              <div style={{ fontSize: '2rem' }}>🧬</div>
              <p>Calcul des générations...</p>
            </div>
          )}

          {result && (
            <div>
              <div style={{ display: 'flex', gap: '20px', marginBottom: '20px' }}>
                <div
                  style={{ background: '#374151', padding: '10px', borderRadius: '6px', flex: 1 }}
                >
                  <div style={{ fontSize: '0.8em', color: '#9ca3af' }}>Meilleur Score</div>
                  <div style={{ fontSize: '1.5em', fontWeight: 'bold', color: '#34d399' }}>
                    {result.best_score}%
                  </div>
                </div>
                <div
                  style={{ background: '#374151', padding: '10px', borderRadius: '6px', flex: 1 }}
                >
                  <div style={{ fontSize: '0.8em', color: '#9ca3af' }}>Durée</div>
                  <div style={{ fontSize: '1.5em', fontWeight: 'bold', color: '#60a5fa' }}>
                    {result.duration_ms} ms
                  </div>
                </div>
                <div
                  style={{ background: '#374151', padding: '10px', borderRadius: '6px', flex: 1 }}
                >
                  <div style={{ fontSize: '0.8em', color: '#9ca3af' }}>Candidat ID</div>
                  <div style={{ fontSize: '1.2em', fontWeight: 'bold', color: '#f472b6' }}>
                    {result.best_candidate_id}
                  </div>
                </div>
              </div>

              <h4>Historique de Convergence</h4>
              {/* Petit graphique CSS simple */}
              <div style={styles.chartContainer}>
                {result.improvement_log.map((val, idx) => (
                  <div
                    key={idx}
                    style={styles.chartBar(val)}
                    title={`Gen ${idx}: ${val.toFixed(1)}%`}
                  ></div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
