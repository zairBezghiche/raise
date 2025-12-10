import { useState } from 'react';
import { codegenService } from '@/services/codegenService';
import { useModelStore } from '@/store/model-store';

export default function CodeGenerator() {
  const [language, setLanguage] = useState('rust');
  const [code, setCode] = useState('// Le code généré apparaîtra ici...');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const currentProject = useModelStore((state) => state.project);

  const handleGenerate = async () => {
    if (!currentProject) {
      setError("Aucun modèle chargé. Veuillez d'abord charger un projet.");
      return;
    }

    setLoading(true);
    setError(null);
    setCopied(false);

    try {
      // On passe une version simplifiée du modèle ou le modèle brut
      // Ici, on envoie le modèle brut, l'adaptateur sera côté Rust si besoin
      const result = await codegenService.generateCode(language, currentProject);
      setCode(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Styles "IDE Dark Mode"
  const styles = {
    container: {
      height: '100%',
      display: 'flex',
      flexDirection: 'column' as const,
      gap: '10px',
      padding: '20px',
      fontFamily: 'Inter, sans-serif',
    },
    toolbar: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      padding: '10px 15px',
      background: '#1e1e1e',
      borderRadius: '8px 8px 0 0',
      borderBottom: '1px solid #333',
    },
    controls: { display: 'flex', gap: '10px', alignItems: 'center' },
    select: {
      background: '#2d2d2d',
      color: '#ccc',
      border: '1px solid #444',
      padding: '6px 12px',
      borderRadius: '4px',
      cursor: 'pointer',
      outline: 'none',
      fontWeight: 'bold' as const,
    },
    btnGen: {
      background: 'var(--color-primary, #4f46e5)',
      color: 'white',
      border: 'none',
      padding: '6px 16px',
      borderRadius: '4px',
      cursor: 'pointer',
      fontWeight: 'bold' as const,
      display: 'flex',
      alignItems: 'center',
      gap: '6px',
    },
    btnCopy: {
      background: copied ? '#10b981' : '#374151',
      color: 'white',
      border: 'none',
      padding: '6px 12px',
      borderRadius: '4px',
      cursor: 'pointer',
      transition: 'background 0.2s',
      fontSize: '0.9em',
    },
    editor: {
      flex: 1,
      background: '#1e1e1e',
      color: '#d4d4d4',
      padding: '20px',
      borderRadius: '0 0 8px 8px',
      overflow: 'auto',
      fontFamily: "'Fira Code', 'Consolas', monospace",
      fontSize: '14px',
      lineHeight: '1.5',
      border: '1px solid #333',
      borderTop: 'none',
      whiteSpace: 'pre' as const,
    },
    status: { fontSize: '0.85em', color: '#6b7280', marginTop: '5px' },
  };

  return (
    <div style={styles.container}>
      <header>
        <h2 style={{ margin: 0, color: 'var(--color-primary)' }}>Usine Logicielle</h2>
        <p style={{ margin: '5px 0 0', color: '#666', fontSize: '0.9em' }}>
          Générez du code infrastructurel à partir de votre modèle d'architecture.
        </p>
      </header>

      {/* --- BARRE D'OUTILS --- */}
      <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>
        <div style={styles.toolbar}>
          <div style={styles.controls}>
            <span style={{ color: '#888', fontSize: '0.9em', fontWeight: 600 }}>Cible :</span>
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              style={styles.select}
            >
              <option value="rust">🦀 Rust (System)</option>
              <option value="python">🐍 Python (Scripting)</option>
              <option value="cpp">⚙️ C++ (Embedded)</option>
            </select>

            <button onClick={handleGenerate} disabled={loading} style={styles.btnGen}>
              {loading ? 'Construction...' : '⚡ Générer'}
            </button>
          </div>

          <button onClick={handleCopy} disabled={!code} style={styles.btnCopy}>
            {copied ? 'Copié !' : '📋 Copier'}
          </button>
        </div>

        {/* --- ZONE ÉDITEUR --- */}
        <div style={styles.editor}>
          {error ? (
            <div style={{ color: '#ef4444' }}>❌ Erreur : {error}</div>
          ) : (
            <code>{code}</code>
          )}
        </div>
      </div>

      <div style={styles.status}>
        Moteur de templates : <strong>Tera</strong> • Modèle actif :{' '}
        <strong>{currentProject ? (currentProject as any).id || 'Inconnu' : 'Aucun'}</strong>
      </div>
    </div>
  );
}
