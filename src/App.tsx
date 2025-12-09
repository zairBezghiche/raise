import { useState, useEffect } from 'react';
import './styles/variables.css';
import './styles/globals.css';

// --- Composants ---
import { ChatInterface } from '@/components/ai-chat/ChatInterface';
import { JsonDbTester } from '@/components/JsonDbTester';
import { DataDictionary } from '@/components/model-viewer/DataDictionary';
import { BlockchainToast } from '@/components/demo/BlockchainToast';

// --- Services & Store ---
import { modelService } from '@/services/model-service'; // <--- Import nécessaire
import { useModelStore } from '@/store/model-store'; // <--- Import nécessaire

// --- Types ---
type ViewId = 'assistant' | 'dictionary' | 'admin-db' | 'settings';

export default function App() {
  const [activeView, setActiveView] = useState<ViewId>('assistant');
  const [triggerBlockchain, setTriggerBlockchain] = useState(false);

  // Récupération de l'action pour mettre à jour le store
  const setProject = useModelStore((state) => state.setProject);

  // --- 1. Chargement Automatique du Modèle au Démarrage ---
  useEffect(() => {
    const initApp = async () => {
      try {
        console.log('🚀 Démarrage : Chargement automatique du modèle...');
        // On charge par défaut l'espace 'un2' et la db '_system'
        const model = await modelService.loadProjectModel('un2', '_system');
        setProject(model);
        console.log("✅ Modèle chargé et injecté dans l'UI.");
      } catch (error) {
        console.error('❌ Échec du chargement automatique :', error);
      }
    };

    initApp();
  }, [setProject]);

  // --- 2. Gestion du "Secret" Blockchain (Touche 'b') ---
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === 'b' && !e.ctrlKey && !e.metaKey) {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

        console.log('🎬 Action : Déclenchement Notification Blockchain');
        setTriggerBlockchain(true);
        setTimeout(() => setTriggerBlockchain(false), 2000);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // --- Rendu de la vue active ---
  const renderContent = () => {
    switch (activeView) {
      case 'assistant':
        return <ChatInterface />;
      case 'dictionary':
        return <DataDictionary />;
      case 'admin-db':
        return <JsonDbTester />;
      case 'settings':
        return (
          <div style={{ padding: 20, color: 'var(--color-gray-500)' }}>
            <h2>Paramètres</h2>
            <p>Configuration de l'application (À venir...)</p>
            <ul>
              <li>
                <a href="/pages/charte-graphique.html" target="_blank" className="text-primary">
                  Voir la Charte Graphique
                </a>
              </li>
              <li>
                <a href="/pages/dark-mode-demo.html" target="_blank" className="text-primary">
                  Démo Mode Sombre
                </a>
              </li>
            </ul>
          </div>
        );
      default:
        return <div>Vue inconnue</div>;
    }
  };

  return (
    <div className="app-shell">
      {/* Toast de démo */}
      <BlockchainToast trigger={triggerBlockchain} />

      {/* 1. BARRE LATÉRALE (Navigation) */}
      <nav className="sidebar">
        <div className="logo-area">
          <img src="/assets/icons/genaptitude-icon.svg" alt="Logo" width="32" height="32" />
        </div>

        <div className="nav-items">
          <NavItem
            id="assistant"
            label="Assistant IA"
            icon="🤖"
            isActive={activeView === 'assistant'}
            onClick={setActiveView}
          />
          <NavItem
            id="dictionary"
            label="Modèle & Data"
            icon="📚"
            isActive={activeView === 'dictionary'}
            onClick={setActiveView}
          />
          <NavItem
            id="admin-db"
            label="Base de Données"
            icon="🛠️"
            isActive={activeView === 'admin-db'}
            onClick={setActiveView}
          />
        </div>

        <div className="nav-footer">
          <NavItem
            id="settings"
            label="Réglages"
            icon="⚙️"
            isActive={activeView === 'settings'}
            onClick={setActiveView}
          />
        </div>
      </nav>

      {/* 2. ZONE PRINCIPALE */}
      <main className="main-content">
        <header className="view-header">
          <h1 className="text-primary">GenAptitude</h1>
          <span className="view-title">
            {activeView === 'assistant' && ' / Assistant Ingénieur'}
            {activeView === 'dictionary' && ' / Dictionnaire de Données'}
            {activeView === 'admin-db' && ' / Administration BDD'}
            {activeView === 'settings' && ' / Paramètres'}
          </span>
        </header>

        <div className="view-body">{renderContent()}</div>
      </main>

      <style>{`
        .app-shell {
          display: flex;
          height: 100vh;
          width: 100vw;
          background-color: var(--color-gray-50);
          color: var(--color-gray-900);
          overflow: hidden;
        }

        .sidebar {
          width: 64px;
          background-color: var(--surface-secondary);
          border-right: 1px solid var(--color-gray-200);
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 16px 0;
          z-index: 10;
          transition: width 0.2s;
        }
        
        .sidebar:hover {
           width: 200px;
           align-items: stretch;
           padding: 16px;
        }

        .logo-area {
          margin-bottom: 32px;
          display: flex;
          justify-content: center;
        }

        .nav-items {
          display: flex;
          flex-direction: column;
          gap: 8px;
          flex: 1;
          width: 100%;
        }

        .nav-footer {
            width: 100%;
        }

        .main-content {
          flex: 1;
          display: flex;
          flex-direction: column;
          min-width: 0;
        }

        .view-header {
          height: 60px;
          border-bottom: 1px solid var(--color-gray-200);
          background-color: var(--surface-primary);
          display: flex;
          align-items: center;
          padding: 0 24px;
          gap: 12px;
        }
        
        .view-header h1 {
            font-size: 1.2rem;
            margin: 0;
        }
        
        .view-title {
            color: var(--color-gray-500);
            font-size: 1rem;
        }

        .view-body {
          flex: 1;
          overflow: hidden;
          padding: 0;
          position: relative;
        }
      `}</style>
    </div>
  );
}

// --- Sous-composant NavItem ---

interface NavItemProps {
  id: ViewId;
  label: string;
  icon: string;
  isActive: boolean;
  onClick: (id: ViewId) => void;
}

function NavItem({ id, label, icon, isActive, onClick }: NavItemProps) {
  return (
    <button
      onClick={() => onClick(id)}
      className={`nav-btn ${isActive ? 'active' : ''}`}
      title={label}
    >
      <span className="icon">{icon}</span>
      <span className="label">{label}</span>

      <style>{`
        .nav-btn {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 10px;
          border: none;
          background: transparent;
          color: var(--color-gray-500);
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s ease;
          width: 100%;
          justify-content: center;
        }
        
        .sidebar:hover .nav-btn {
            justify-content: flex-start;
            padding: 10px 16px;
        }

        .nav-btn:hover {
          background-color: var(--color-gray-200);
          color: var(--color-gray-900);
        }

        .nav-btn.active {
          background-color: var(--color-primary-light);
          color: var(--color-white);
          background: var(--gradient-primary);
        }

        .icon {
          font-size: 1.2rem;
          line-height: 1;
        }

        .label {
          font-size: 0.9rem;
          font-weight: 500;
          white-space: nowrap;
          display: none;
        }

        .sidebar:hover .label {
            display: block;
        }
      `}</style>
    </button>
  );
}
