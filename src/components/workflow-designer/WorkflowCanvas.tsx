import { useState, useEffect, DragEvent } from 'react'; // CORRECTION : useRef retiré
import { invoke } from '@tauri-apps/api/core';

import { NodeLibrary } from './NodeLibrary';
import { ConnectionManager } from './ConnectionManager';
import { ExecutionMonitor } from './ExecutionMonitor';

// --- TYPES RUST ---
type ExecutionStatus = 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'PAUSED' | 'SKIPPED';

interface WorkflowView {
  id: string;
  status: ExecutionStatus;
  current_nodes: string[];
  logs: string[];
}

// --- DÉMO DATA ---
const INITIAL_NODES = [
  { id: 'step-1', type: 'task', label: '🔍 Analyse IA', x: 100, y: 100 },
  { id: 'step-2', type: 'gate_hitl', label: '🛡️ Validation Humaine', x: 400, y: 100 },
  { id: 'step-3', type: 'task', label: '📦 Compilation', x: 400, y: 300 },
  { id: 'step-4', type: 'end', label: '🚀 Déploiement', x: 700, y: 300 },
];

const INITIAL_CONNECTIONS = [
  { id: 'c1', from: 'step-1', to: 'step-2' },
  { id: 'c2', from: 'step-2', to: 'step-3' },
  { id: 'c3', from: 'step-3', to: 'step-4' },
];

export default function WorkflowCanvas() {
  // État visuel
  const [nodes, setNodes] = useState(INITIAL_NODES);

  // CORRECTION : On retire 'setConnections' car on ne modifie pas les liens dans cette démo
  const [connections] = useState(INITIAL_CONNECTIONS);

  // État Backend
  const [instance, setInstance] = useState<WorkflowView | null>(null);
  const [pollingId, setPollingId] = useState<number | null>(null);

  // --- LIFECYCLE ---
  useEffect(() => {
    return () => stopPolling();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pollingId]);

  const stopPolling = () => {
    if (pollingId) window.clearInterval(pollingId);
  };

  const startPolling = (instanceId: string) => {
    stopPolling();
    const pid = window.setInterval(async () => {
      try {
        const view: WorkflowView = await invoke('get_workflow_state', { instanceId });
        setInstance(view);

        if (view.status === 'COMPLETED' || view.status === 'FAILED') {
          window.clearInterval(pid);
        }
      } catch (e) {
        console.error('Erreur polling', e);
      }
    }, 500);
    setPollingId(pid);
  };

  // --- ACTIONS ---

  const handleRun = async () => {
    try {
      // Construction de la définition pour le Backend
      const definition = {
        id: 'demo-pipeline',
        entry: 'step-1',
        nodes: nodes.map((n) => ({
          id: n.id,
          type: n.type === 'trigger' ? 'task' : n.type,
          name: n.label,
          params: {},
        })),
        edges: connections.map((c) => ({ from: c.from, to: c.to })),
      };

      console.log('Enregistrement...', definition);
      await invoke('register_workflow', { definition });

      console.log('Démarrage...');
      const view: WorkflowView = await invoke('start_workflow', { workflowId: definition.id });
      setInstance(view);
      startPolling(view.id);
    } catch (err) {
      console.error('Erreur Backend:', err);
      alert('Erreur Backend: ' + err);
    }
  };

  const handleResume = async (nodeId: string, approved: boolean) => {
    if (!instance) return;
    try {
      await invoke('resume_workflow', {
        instanceId: instance.id,
        nodeId,
        approved,
      });
      // Le polling mettra à jour l'UI automatiquement
    } catch (err) {
      alert('Erreur resume: ' + err);
    }
  };

  // --- DRAG & DROP ---
  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    const type = e.dataTransfer.getData('workflowNodeType');
    if (type) {
      const rect = e.currentTarget.getBoundingClientRect();
      const newNode = {
        id: `node-${Date.now()}`,
        type,
        label: `Nouveau ${type}`,
        x: e.clientX - rect.left - 75,
        y: e.clientY - rect.top - 30,
      };
      setNodes([...nodes, newNode]);
    }
  };
  const handleDragOver = (e: DragEvent) => e.preventDefault();

  // --- HELPER RENDU ---
  const getNodeColor = (node: any) => {
    if (!instance) return 'var(--bg-panel)';

    // Si le nœud est actif
    if (instance.current_nodes.includes(node.id)) {
      if (instance.status === 'PAUSED') return 'var(--color-warning)';
      return 'var(--color-info)';
    }

    // Si le nœud est terminé
    const nodeIndex = nodes.findIndex((n) => n.id === node.id);
    const activeIndex = nodes.findIndex((n) => instance.current_nodes.includes(n.id));

    if (instance.status === 'COMPLETED') return 'var(--color-success)';
    if (activeIndex > -1 && nodeIndex < activeIndex) return 'var(--color-success)';

    return 'var(--bg-panel)';
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Zone principale */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <NodeLibrary />

        <div
          style={{
            flex: 1,
            position: 'relative',
            backgroundColor: 'var(--bg-app)',
            backgroundImage: 'radial-gradient(var(--border-color) 1px, transparent 1px)',
            backgroundSize: '20px 20px',
            overflow: 'hidden',
          }}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        >
          <ConnectionManager nodes={nodes} connections={connections} />

          {nodes.map((node) => {
            const isPausedHere =
              instance?.status === 'PAUSED' && instance.current_nodes.includes(node.id);
            const borderColor = getNodeColor(node);

            return (
              <div
                key={node.id}
                style={{
                  position: 'absolute',
                  left: node.x,
                  top: node.y,
                  width: '180px',
                  padding: 'var(--spacing-3)',
                  backgroundColor: 'var(--bg-panel)',
                  border: `2px solid ${borderColor}`,
                  borderRadius: 'var(--radius-md)',
                  boxShadow: 'var(--shadow-md)',
                  zIndex: 5,
                  cursor: 'move',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                  transition: 'border-color 0.3s ease',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <div
                    style={{
                      width: '10px',
                      height: '10px',
                      borderRadius: '50%',
                      backgroundColor:
                        node.type === 'end' ? 'var(--color-error)' : 'var(--color-primary)',
                    }}
                  />
                  <span
                    style={{
                      fontSize: 'var(--font-size-sm)',
                      fontWeight: 'bold',
                      color: 'var(--text-main)',
                    }}
                  >
                    {node.label}
                  </span>
                </div>

                {/* BOUTONS HITL */}
                {isPausedHere && (
                  <div style={{ display: 'flex', gap: '4px', marginTop: '4px' }}>
                    <button
                      onClick={() => handleResume(node.id, true)}
                      style={{
                        flex: 1,
                        background: 'var(--color-success)',
                        border: 0,
                        color: 'white',
                        borderRadius: 4,
                        cursor: 'pointer',
                        fontSize: 10,
                        padding: 4,
                      }}
                    >
                      Valider
                    </button>
                    <button
                      onClick={() => handleResume(node.id, false)}
                      style={{
                        flex: 1,
                        background: 'var(--color-error)',
                        border: 0,
                        color: 'white',
                        borderRadius: 4,
                        cursor: 'pointer',
                        fontSize: 10,
                        padding: 4,
                      }}
                    >
                      Rejeter
                    </button>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Panneau inférieur connecté au Backend */}
      <ExecutionMonitor
        logs={instance?.logs || []}
        status={instance?.status || 'PRÊT'}
        onStart={handleRun}
        isRunning={instance?.status === 'RUNNING'}
      />
    </div>
  );
}
