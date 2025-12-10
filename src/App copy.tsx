import { useState, useEffect } from 'react';
import './styles/variables.css';
import './styles/globals.css';

// --- Composants Existants ---
import { ChatInterface } from '@/components/ai-chat/ChatInterface';
import { JsonDbTester } from '@/components/JsonDbTester';
import { DataDictionary } from '@/components/model-viewer/DataDictionary';
import { BlockchainToast } from '@/components/demo/BlockchainToast';
import CognitiveTester from '@/components/CognitiveTester';

// --- Services & Store ---
import { modelService } from '@/services/model-service';
import { useModelStore } from '@/store/model-store';

// --- Types ---
// Mise à jour de la liste complète des vues
type ViewId =
  | 'assistant'
  | 'dictionary'
  | 'cognitive'
  | 'blockchain' // Nouveau
  | 'codegen' // Nouveau
  | 'genetics' // Nouveau
  | 'admin-db'
  | 'settings';

export default function App() {
  const [activeView, setActiveView] = useState<ViewId>('assistant');
  const [triggerBlockchain, setTriggerBlockchain] = useState(false);

  const setProject = useModelStore((state) => state.setProject);

  // --- 1. Chargement Automatique ---
  useEffect(() => {
    const initApp = async () => {
      try {
        console.log('🚀 Démarrage : Chargement automatique du modèle...');
        const model = await modelService.loadProjectModel('un2', '_system');
        setProject(model);
        console.log('✅ Modèle chargé.');
      } catch (error) {
        console.error('❌ Échec du chargement automatique :', error);
      }
    };
    initApp();
  }, [setProject]);

  // --- 2. Raccourci Clavier Blockchain (Touche 'b') ---
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === 'b' && !e.ctrlKey && !e.metaKey) {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
        setTriggerBlockchain(true);
        setTimeout(() => setTriggerBlockchain(false), 3000);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // --- Rendu du Contenu Central ---
  const renderContent = () => {
    switch (activeView) {
      // 1. Module AI
      case 'assistant':
        return <ChatInterface />;

      // 2. Module Model Engine
      case 'dictionary':
        return <DataDictionary />;

      // 3. Module Plugins (WASM)
      case 'cognitive':
        return <CognitiveTester />;

      // 4. Module JsonDB
      case 'admin-db':
        return <JsonDbTester />;

      // --- NOUVEAUX MODULES (Placeholders) ---

      // 5. Module Blockchain
      case 'blockchain':
        return (
          <div className="placeholder-container">
            <div className="placeholder-icon">🔗</div>
            <h2>Blockchain & Réseau</h2>
            <p>
              Module : <code>src-tauri/src/blockchain</code>
            </p>
            <p className="description">
              Visualisation des pairs VPN, état du registre Hyperledger Fabric et traçabilité des
              transactions.
            </p>
            <button className="action-btn" onClick={() => setTriggerBlockchain(true)}>
              Tester la notification Toast
            </button>
          </div>
        );

      // 6. Module Code Generator
      case 'codegen':
        return (
          <div className="placeholder-container">
            <div className="placeholder-icon">⚡</div>
            <h2>Générateur de Code</h2>
            <p>
              Module : <code>src-tauri/src/code_generator</code>
            </p>
            <p className="description">
              Moteur de templates Tera pour la génération de code source (Rust, Python, C++) à
              partir du modèle d'architecture.
            </p>
          </div>
        );

      // 7. Module Genetics
      case 'genetics':
        return (
          <div className="placeholder-container">
            <div className="placeholder-icon">🧬</div>
            <h2>Optimisation Génétique</h2>
            <p>
              Module : <code>src-tauri/src/genetics</code>
            </p>
            <p className="description">
              Algorithmes évolutionnaires pour l'exploration de l'espace de conception et
              l'optimisation multi-critères.
            </p>
          </div>
        );

      case 'settings':
        return (
          <div className="placeholder-container">
            <h2>Paramètres</h2>
            <p>Configuration globale de GenAptitude.</p>
          </div>
        );

      default:
        return <div>Vue inconnue</div>;
    }
  };

  return (
    <div className="app-shell">
      <BlockchainToast trigger={triggerBlockchain} />

      {/* --- BARRE LATÉRALE --- */}
      <nav className="sidebar">
        <div className="logo-area">
          <img src="/assets/icons/genaptitude-icon.svg" alt="G" width="32" height="32" />
        </div>

        <div className="nav-items">
          <div className="nav-group-label">Intelligence</div>
          <NavItem
            id="assistant"
            label="Assistant IA"
            icon="🤖"
            isActive={activeView === 'assistant'}
            onClick={setActiveView}
          />
          <NavItem
            id="cognitive"
            label="Blocs Cognitifs"
            icon="🧠"
            isActive={activeView === 'cognitive'}
            onClick={setActiveView}
          />

          <div className="nav-group-label">Ingénierie</div>
          <NavItem
            id="dictionary"
            label="Modèle & Data"
            icon="📚"
            isActive={activeView === 'dictionary'}
            onClick={setActiveView}
          />
          <NavItem
            id="codegen"
            label="Générateur Code"
            icon="⚡"
            isActive={activeView === 'codegen'}
            onClick={setActiveView}
          />
          <NavItem
            id="genetics"
            label="Génétique"
            icon="🧬"
            isActive={activeView === 'genetics'}
            onClick={setActiveView}
          />

          <div className="nav-group-label">Infrastructure</div>
          <NavItem
            id="blockchain"
            label="Blockchain"
            icon="🔗"
            isActive={activeView === 'blockchain'}
            onClick={setActiveView}
          />
          <NavItem
            id="admin-db"
            label="Base de Données"
            icon="🗄️"
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

      {/* --- ZONE PRINCIPALE --- */}
      <main className="main-content">
        <header className="view-header">
          <h1 className="text-primary">GenAptitude</h1>
          <span className="view-title">
            {activeView === 'assistant' && ' / Assistant Ingénieur'}
            {activeView === 'dictionary' && ' / Dictionnaire de Données'}
            {activeView === 'cognitive' && ' / Moteur Cognitif (WASM)'}
            {activeView === 'blockchain' && ' / Réseau & Traçabilité'}
            {activeView === 'codegen' && ' / Usine Logicielle'}
            {activeView === 'genetics' && ' / Optimisation Système'}
            {activeView === 'admin-db' && ' / Administration BDD'}
            {activeView === 'settings' && ' / Paramètres'}
          </span>
        </header>

        <div className="view-body">{renderContent()}</div>
      </main>

      {/* --- STYLES GLOBAUX DU LAYOUT --- */}
      <style>{`
        .app-shell {
          display: flex;
          height: 100vh;
          width: 100vw;
          background-color: var(--color-gray-50);
          color: var(--color-gray-900);
          overflow: hidden;
        }

        /* Sidebar Styling */
        .sidebar {
          width: 70px; /* Un peu plus large pour les icônes */
          background-color: var(--surface-secondary);
          border-right: 1px solid var(--color-gray-200);
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 16px 0;
          z-index: 10;
          transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        }
        
        .sidebar:hover {
           width: 240px; /* Assez large pour lire les titres */
           align-items: stretch;
           padding: 16px;
        }

        .logo-area {
          margin-bottom: 24px;
          display: flex;
          justify-content: center;
          height: 40px;
          align-items: center;
        }

        .nav-items {
          display: flex;
          flex-direction: column;
          gap: 4px;
          flex: 1;
          width: 100%;
          overflow-y: auto;
        }

        .nav-group-label {
          font-size: 0.75rem;
          text-transform: uppercase;
          color: var(--color-gray-400);
          margin: 16px 0 8px 12px;
          font-weight: 600;
          display: none; /* Caché quand replié */
          white-space: nowrap;
        }

        .sidebar:hover .nav-group-label {
          display: block;
        }

        .nav-footer {
            width: 100%;
            border-top: 1px solid var(--color-gray-200);
            padding-top: 8px;
            margin-top: 8px;
        }

        /* Main Content Styling */
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
        
        .view-header h1 { font-size: 1.2rem; margin: 0; font-weight: 700; }
        .view-title { color: var(--color-gray-500); font-size: 1rem; }

        .view-body {
          flex: 1;
          overflow: auto;
          padding: 0;
          position: relative;
        }

        /* Placeholder Views Styling */
        .placeholder-container {
          padding: 40px;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: var(--color-gray-500);
          text-align: center;
        }
        .placeholder-icon { font-size: 4rem; margin-bottom: 20px; opacity: 0.5; }
        .description { max-width: 500px; margin-top: 10px; line-height: 1.5; }
        .action-btn {
          margin-top: 20px;
          padding: 10px 20px;
          background: var(--color-primary);
          color: white;
          border: none;
          border-radius: 6px;
          cursor: pointer;
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
          padding: 10px 12px;
          border: none;
          background: transparent;
          color: var(--color-gray-500);
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s ease;
          width: 100%;
          justify-content: center; /* Centré quand replié */
          height: 44px;
        }
        
        .sidebar:hover .nav-btn {
            justify-content: flex-start; /* Aligné gauche quand déplié */
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

        .icon { font-size: 1.2rem; line-height: 1; min-width: 24px; text-align: center; }
        
        .label {
          font-size: 0.9rem;
          font-weight: 500;
          white-space: nowrap;
          display: none; /* Caché quand replié */
          opacity: 0;
          animation: fadeIn 0.3s forwards;
        }

        .sidebar:hover .label { display: block; }
        
        @keyframes fadeIn { to { opacity: 1; } }
      `}</style>
    </button>
  );
}
