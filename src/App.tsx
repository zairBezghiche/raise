import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './styles/globals.css';

// --- UTILS ---
import { MOCK_PROJECT } from '@/utils/mock-data';

// --- STORES ---
import { useModelStore } from '@/store/model-store';

// --- LAYOUT ---
import { MainLayout } from '@/components/layout/MainLayout';

// --- VUES / MODULES ---
import CapellaViewer from '@/components/model-viewer/CapellaViewer';
import GeneticsDashboard from '@/components/genetics/GeneticsDashboard';
import CodeGenerator from '@/components/codegen/CodeGenerator';
import DiagramCanvas from '@/components/diagram-editor/DiagramCanvas';
import WorkflowCanvas from '@/components/workflow-designer/WorkflowCanvas';
import CognitiveAnalysis from '@/components/cognitive/CognitiveAnalysis';
import AssuranceDashboard from '@/components/assurance/AssuranceDashboard';
import MBAIEView from '@/components/ai-chat/MBAIEView';
import SettingsPage from '@/components/settings/SettingsPage';
import { JsonDbTester } from '@/components/JsonDbTester';
import CognitiveTester from '@/components/CognitiveTester';

// --- VUES REFACTORISÉES ---
import RulesEngineDashboard from '@/components/rules_engine/RulesEngineDashboard';
import BlockchainView from '@/components/blockchain/BlockchainView';
import DashboardView from '@/components/dashboard/DashboardView';

export default function App() {
  const [currentPage, setCurrentPage] = useState('dashboard');
  const [sysInfo, setSysInfo] = useState<any>(null);
  const { setProject } = useModelStore();

  // --- BOOTSTRAP ---
  useEffect(() => {
    console.log('🚀 Démarrage de GenAptitude (Frontend + Tauri)...');

    invoke('get_app_info')
      .then((response) => setSysInfo(response))
      .catch((error) => console.error('❌ Erreur Backend Rust :', error));

    const timer = setTimeout(() => {
      setProject(MOCK_PROJECT);
    }, 500);
    return () => clearTimeout(timer);
  }, [setProject]);

  // --- ROUTING ---
  const renderContent = () => {
    switch (currentPage) {
      // Vues Métier
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
      case 'cognitive':
        return <CognitiveAnalysis />;
      case 'assurance':
        return <AssuranceDashboard />;
      case 'ai':
        return <MBAIEView />;

      // Outils Système
      case 'settings':
        return <SettingsPage />;
      case 'admin-db':
        return <JsonDbTester />;
      case 'cognitive-tester':
        return <CognitiveTester />;

      // Vues Démo / Complexes refactorisées
      case 'rules-engine':
        return <RulesEngineDashboard />;
      case 'blockchain':
        return <BlockchainView />;

      case 'dashboard':
      default:
        return <DashboardView sysInfo={sysInfo} onNavigate={setCurrentPage} />;
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
      case 'rules-engine':
        return 'Moteur de Règles (GenRules)';
      default:
        return 'GenAptitude';
    }
  };

  return (
    <MainLayout currentPage={currentPage} onNavigate={setCurrentPage} pageTitle={getTitle()}>
      {renderContent()}
    </MainLayout>
  );
}
