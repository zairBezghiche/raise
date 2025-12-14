import { useModelStore } from '@/store/model-store';

interface DashboardViewProps {
  sysInfo: any;
  onNavigate: (page: string) => void;
}

export default function DashboardView({ sysInfo, onNavigate }: DashboardViewProps) {
  const { project } = useModelStore();

  return (
    <div style={{ padding: 'var(--spacing-8)', color: 'var(--text-main)' }}>
      <h1 style={{ fontSize: 'var(--font-size-3xl)', marginBottom: 'var(--spacing-6)' }}>
        Tableau de Bord
      </h1>

      {/* CARTES KPI */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
          gap: 'var(--spacing-4)',
        }}
      >
        <DashboardCard
          title="Projet Actif"
          value={project?.meta?.name || 'Aucun'}
          icon="💠"
          desc={project?.meta?.description || 'Chargement...'}
        />
        <DashboardCard
          title="Éléments"
          value={project ? String(project.meta?.elementCount || 42) : '-'}
          icon="📊"
          desc="Objets indexés en mémoire"
        />
        <DashboardCard
          title="Moteur IA"
          value="Connecté"
          icon="⚡"
          desc="Backend Rust opérationnel"
        />
      </div>

      {/* INFO SYSTÈME RUST */}
      {sysInfo && (
        <div
          style={{
            marginTop: 'var(--spacing-8)',
            padding: 'var(--spacing-4)',
            backgroundColor: 'var(--bg-panel)',
            border: '1px solid var(--color-success)',
            borderRadius: 'var(--radius-lg)',
            color: 'var(--text-muted)',
            fontSize: 'var(--font-size-sm)',
            fontFamily: 'var(--font-family-mono)',
          }}
        >
          <h3 style={{ marginTop: 0, color: 'var(--color-success)', fontSize: '1rem' }}>
            ✅ Backend Rust Connecté
          </h3>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '10px',
              marginTop: '10px',
            }}
          >
            <div>
              <strong>Version :</strong> v{sysInfo.app_version}
            </div>
            <div>
              <strong>Environnement :</strong> {sysInfo.env_mode}
            </div>
            <div>
              <strong>Base de Données :</strong> {sysInfo.database_path}
            </div>
            <div>
              <strong>API :</strong> {sysInfo.api_status}
            </div>
          </div>
        </div>
      )}

      {/* BARRE D'ACTIONS RAPIDES */}
      <div style={{ marginTop: 'var(--spacing-8)', display: 'flex', gap: '10px' }}>
        <ActionButton
          onClick={() => onNavigate('settings')}
          label="⚙️ Paramètres"
          primary={false}
        />
        <ActionButton
          onClick={() => onNavigate('rules-engine')}
          label="🧮 Démo Règles (GenRules)"
          primary={true}
        />
      </div>
    </div>
  );
}

// --- SOUS-COMPOSANTS LOCAUX ---

function DashboardCard({ title, value, icon, desc }: any) {
  return (
    <div
      style={{
        backgroundColor: 'var(--bg-panel)',
        border: '1px solid var(--border-color)',
        borderRadius: 'var(--radius-lg)',
        padding: 'var(--spacing-6)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--spacing-2)',
        boxShadow: 'var(--shadow-sm)',
        transition: 'transform 0.2s',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3
          style={{
            margin: 0,
            color: 'var(--text-muted)',
            fontSize: 'var(--font-size-sm)',
            textTransform: 'uppercase',
          }}
        >
          {title}
        </h3>
        <span style={{ fontSize: '1.5rem' }}>{icon}</span>
      </div>
      <div style={{ fontSize: '1.8rem', fontWeight: 'bold', color: 'var(--text-main)' }}>
        {value}
      </div>
      <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--text-muted)' }}>{desc}</div>
    </div>
  );
}

function ActionButton({ onClick, label, primary }: any) {
  return (
    <button
      onClick={onClick}
      style={{
        color: primary ? '#fff' : 'var(--color-primary)',
        background: primary ? 'var(--color-primary)' : 'transparent',
        border: primary ? 'none' : '1px solid var(--color-primary)',
        padding: '8px 16px',
        borderRadius: 'var(--radius-sm)',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontWeight: primary ? 'bold' : 'normal',
        boxShadow: primary ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
      }}
    >
      {label}
    </button>
  );
}
