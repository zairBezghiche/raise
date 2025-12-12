import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './styles/globals.css';

// --- TYPES & UTILS ---
import { MOCK_PROJECT } from '@/utils/mock-data';

// --- STORES ---
import { useModelStore } from '@/store/model-store';
// On supprime useUiStore car 'theme' n'était pas utilisé

// --- LAYOUT ---
import { MainLayout } from '@/components/layout/MainLayout';

// --- MODULES ---
import CapellaViewer from '@/components/model-viewer/CapellaViewer';
import GeneticsDashboard from '@/components/genetics/GeneticsDashboard';
import CodeGenerator from '@/components/codegen/CodeGenerator';
import DiagramCanvas from '@/components/diagram-editor/DiagramCanvas';
import WorkflowCanvas from '@/components/workflow-designer/WorkflowCanvas';
import { BlockchainToast } from '@/components/blockchain/BlockchainToast';
import CognitiveAnalysis from '@/components/cognitive/CognitiveAnalysis';
import AssuranceDashboard from '@/components/assurance/AssuranceDashboard';
import MBAIEView from '@/components/ai-chat/MBAIEView';

import SettingsPage from '@/components/settings/SettingsPage';

import { JsonDbTester } from '@/components/JsonDbTester';
import CognitiveTester from '@/components/CognitiveTester';

export default function App() {
  const [currentPage, setCurrentPage] = useState('dashboard');
  const [showBlockchainToast, setShowBlockchainToast] = useState(false);
  const [sysInfo, setSysInfo] = useState<any>(null);
  const { project, setProject } = useModelStore();

  // --- BOOTSTRAP ---
  useEffect(() => {
    console.log('🚀 Démarrage de GenAptitude (Frontend + Tauri)...');
    // AJOUT 2 : Appel à la commande Rust
    // 'get_app_info' doit correspondre exactement au nom de la fonction dans utils_commands.rs
    invoke('get_app_info')
      .then((response) => {
        console.log('✅ Réponse du Backend Rust :', response);
        setSysInfo(response); // On stocke la réponse
      })
      .catch((error) => {
        console.error('❌ Erreur Backend Rust :', error);
      });
    const timer = setTimeout(() => {
      console.log('📦 Chargement du projet Mock (Démo)...');
      setProject(MOCK_PROJECT);
    }, 500);
    return () => clearTimeout(timer);
  }, [setProject]);

  // --- ROUTING ---
  const renderContent = () => {
    switch (currentPage) {
      case 'model':
        return <CapellaViewer />;
      case 'genetics':
        return <GeneticsDashboard />;
      case 'codegen':
        return <CodeGenerator />;
      case 'diagram':
        return <DiagramCanvas />;
      case 'workflow':
        return <WorkflowCanvas />;
      case 'settings':
        return <SettingsPage />; // Utilisation du composant importé
      case 'admin-db':
        return <JsonDbTester />;
      case 'cognitive-tester':
        return <CognitiveTester />;
      case 'ai':
        return <MBAIEView />;

      case 'blockchain':
        return (
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              textAlign: 'center',
              color: 'var(--text-main)',
              gap: 'var(--spacing-4)',
            }}
          >
            <div style={{ fontSize: '4rem', marginBottom: 'var(--spacing-2)' }}>🔗</div>
            <h2 style={{ fontSize: 'var(--font-size-2xl)' }}>Blockchain Ledger Demo</h2>
            <p style={{ maxWidth: 500, color: 'var(--text-muted)', lineHeight: '1.6' }}>
              Cette interface simule l'interaction avec le backend Rust connecté à{' '}
              <strong>Hyperledger Fabric</strong>.
            </p>
            <button
              onClick={() => {
                setShowBlockchainToast(true);
                setTimeout(() => setShowBlockchainToast(false), 3000);
              }}
              style={{
                padding: '12px 24px',
                backgroundColor: 'var(--color-primary)',
                color: 'white',
                border: 'none',
                borderRadius: 'var(--radius-md)',
                cursor: 'pointer',
                fontSize: '1rem',
                fontWeight: 'bold',
                boxShadow: 'var(--shadow-md)',
              }}
            >
              Ancrer une Preuve
            </button>
          </div>
        );
      case 'cognitive':
        return <CognitiveAnalysis />;
      case 'assurance':
        return <AssuranceDashboard />;
      case 'dashboard':
      default:
        return (
          <div style={{ padding: 'var(--spacing-8)', color: 'var(--text-main)' }}>
            <h1 style={{ fontSize: 'var(--font-size-3xl)', marginBottom: 'var(--spacing-6)' }}>
              Tableau de Bord
            </h1>
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
            {/* Affichage des infos système Rust --- */}
            {sysInfo && (
              <div
                style={{
                  marginTop: 'var(--spacing-8)',
                  padding: 'var(--spacing-4)',
                  backgroundColor: 'var(--bg-panel)',
                  border: '1px solid var(--color-success)', // Bordure verte pour dire "Connecté"
                  borderRadius: 'var(--radius-lg)',
                  color: 'var(--text-muted)',
                  fontSize: 'var(--font-size-sm)',
                  fontFamily: 'var(--font-family-mono)',
                }}
              >
                <h3 style={{ marginTop: 0, color: 'var(--color-success)', fontSize: '1rem' }}>
                  ✅ Backend Rust Connecté
                </h3>
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
            )}
            <div style={{ marginTop: 'var(--spacing-8)' }}>
              <button
                onClick={() => setCurrentPage('settings')}
                style={{
                  color: 'var(--color-primary)',
                  background: 'transparent',
                  border: '1px solid var(--color-primary)',
                  padding: '8px 16px',
                  borderRadius: 'var(--radius-sm)',
                  cursor: 'pointer',
                }}
              >
                ⚙️ Ouvrir les Paramètres
              </button>
            </div>
          </div>
        );
    }
  };

  const getTitle = () => {
    switch (currentPage) {
      case 'model':
        return 'Modélisation Arcadia';
      case 'genetics':
        return 'Optimisation Génétique';
      case 'codegen':
        return 'Génération de Code';
      case 'ai':
        return 'MBAIE';
      case 'diagram':
        return 'Éditeur de Diagrammes';
      case 'workflow':
        return 'Workflow Designer';
      case 'blockchain':
        return 'Blockchain Ledger';
      case 'cognitive':
        return 'Blocs Cognitifs';
      case 'assurance':
        return 'Product Assurance & XAI';
      case 'settings':
        return 'Paramètres Système';
      case 'admin-db':
        return 'Gestion de la DB';

      case 'cognitive-tester':
        return 'Diagnostic Cognitif (WASM)';
      default:
        return 'GenAptitude';
    }
  };

  return (
    <MainLayout currentPage={currentPage} onNavigate={setCurrentPage} pageTitle={getTitle()}>
      {renderContent()}
      <BlockchainToast trigger={showBlockchainToast} />
    </MainLayout>
  );
}

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
