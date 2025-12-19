import { useState, useEffect, DragEvent } from 'react';
import { useAiStore } from '@/store/ai-store';
import { CreatedArtifact } from '@/types/ai.types';

import { ShapeLibrary } from './ShapeLibrary';
import { ConnectionTool, ToolType } from './ConnectionTool';
import { LayoutEngine } from './LayoutEngine';

// --- TYPES LOCAUX ---
interface DiagramNode {
  id: string;
  type: string;
  label: string;
  layer?: string;
  x: number;
  y: number;
}

interface DiagramEdge {
  id: string;
  from: string;
  to: string;
}

export default function DiagramCanvas() {
  const { messages } = useAiStore();

  // États du Diagramme
  const [nodes, setNodes] = useState<DiagramNode[]>([]);
  const [edges, setEdges] = useState<DiagramEdge[]>([]);
  const [activeTool, setActiveTool] = useState<ToolType>('select');

  // État pour la création de lien (Fil d'Ariane)
  const [connectingSource, setConnectingSource] = useState<string | null>(null);
  const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(null);

  // --- 1. SYNC IA -> DIAGRAMME ---
  useEffect(() => {
    const existingIds = new Set(nodes.map((n) => n.id));
    const newNodes: DiagramNode[] = [];
    let offset = 0;

    messages.forEach((msg) => {
      if (msg.artifacts) {
        msg.artifacts.forEach((art: CreatedArtifact) => {
          if (!existingIds.has(art.id)) {
            newNodes.push({
              id: art.id,
              type: 'artifact',
              label: art.name,
              layer: art.layer,
              x: 100 + offset * 20,
              y: 100 + offset * 60,
            });
            existingIds.add(art.id);
            offset++;
          }
        });
      }
    });

    if (newNodes.length > 0) {
      setNodes((prev) => [...prev, ...newNodes]);
    }
  }, [messages, nodes]);

  // --- 2. GESTION CLAVIER (Echap pour annuler) ---
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setConnectingSource(null);
        setMousePos(null);
        setActiveTool('select');
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // --- 3. DROP & DRAG ---
  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    const type = e.dataTransfer.getData('shapeType');
    if (type) {
      const rect = e.currentTarget.getBoundingClientRect();
      setNodes([
        ...nodes,
        {
          id: `shape-${Date.now()}`,
          type: 'shape',
          label: type.toUpperCase(),
          layer: 'TRANSVERSE',
          x: e.clientX - rect.left,
          y: e.clientY - rect.top,
        },
      ]);
    }
  };

  const handleDragOver = (e: DragEvent) => e.preventDefault();

  // --- 4. SUIVI SOURIS (Pour le fil d'Ariane) ---
  const handleMouseMove = (e: React.MouseEvent) => {
    if (connectingSource) {
      const rect = e.currentTarget.getBoundingClientRect();
      setMousePos({
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      });
    }
  };

  // --- 5. CLIC SUR NOEUD ---
  const handleNodeClick = (nodeId: string) => {
    // Mode Suppression
    if (activeTool === 'delete') {
      setNodes(nodes.filter((n) => n.id !== nodeId));
      setEdges(edges.filter((e) => e.from !== nodeId && e.to !== nodeId));
      return;
    }

    // Mode Lien
    if (activeTool === 'connect') {
      if (!connectingSource) {
        // Premier clic : on définit la source
        setConnectingSource(nodeId);
      } else {
        // Second clic : on crée le lien
        if (nodeId !== connectingSource) {
          setEdges([
            ...edges,
            {
              id: `edge-${Date.now()}`,
              from: connectingSource,
              to: nodeId,
            },
          ]);
        }
        // Reset après création
        setConnectingSource(null);
        setMousePos(null);
      }
    }
  };

  const layerColors: Record<string, string> = {
    OA: '#eab308',
    SA: '#a855f7',
    LA: '#3b82f6',
    PA: '#22c55e',
    EPBS: '#f97316',
    DATA: '#ef4444',
    TRANSVERSE: '#64748b',
  };

  return (
    <div
      style={{
        display: 'flex',
        height: '100%',
        width: '100%',
        backgroundColor: 'var(--bg-app)',
        overflow: 'hidden',
      }}
    >
      <ShapeLibrary />

      <div
        style={{
          position: 'relative',
          flex: 1,
          height: '100%',
          overflow: 'hidden',
          cursor: activeTool === 'connect' ? 'crosshair' : 'default',
        }}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onMouseMove={handleMouseMove}
      >
        {/* Fond Grille */}
        <div
          style={{
            position: 'absolute',
            inset: 0,
            opacity: 0.1,
            pointerEvents: 'none',
            backgroundImage: `linear-gradient(var(--text-main) 1px, transparent 1px), linear-gradient(90deg, var(--text-main) 1px, transparent 1px)`,
            backgroundSize: '20px 20px',
          }}
        />

        <ConnectionTool
          activeTool={activeTool}
          onToolChange={(tool) => {
            setActiveTool(tool);
            setConnectingSource(null);
          }}
        />
        <LayoutEngine />

        {/* --- COUCHE SVG (Liens) --- */}
        <svg
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            pointerEvents: 'none',
            zIndex: 5,
          }}
        >
          <defs>
            <marker
              id="arrowhead"
              markerWidth="10"
              markerHeight="7"
              refX="10"
              refY="3.5"
              orient="auto"
            >
              <polygon points="0 0, 10 3.5, 0 7" fill="var(--text-muted)" />
            </marker>
          </defs>

          {/* Liens existants */}
          {edges.map((edge) => {
            const source = nodes.find((n) => n.id === edge.from);
            const target = nodes.find((n) => n.id === edge.to);
            if (!source || !target) return null;
            return (
              <line
                key={edge.id}
                x1={source.x + 60}
                y1={source.y + 35}
                x2={target.x + 60}
                y2={target.y + 35}
                stroke="var(--text-muted)"
                strokeWidth="2"
                markerEnd="url(#arrowhead)"
              />
            );
          })}

          {/* Ligne élastique (Fil d'Ariane) en cours de création */}
          {connectingSource &&
            mousePos &&
            (() => {
              const source = nodes.find((n) => n.id === connectingSource);
              if (!source) return null;
              return (
                <line
                  x1={source.x + 60}
                  y1={source.y + 35}
                  x2={mousePos.x}
                  y2={mousePos.y}
                  stroke="var(--color-primary)"
                  strokeWidth="2"
                  strokeDasharray="5,5"
                />
              );
            })()}
        </svg>

        {/* --- COUCHE NOEUDS --- */}
        {nodes.map((node) => {
          const color = layerColors[node.layer || 'TRANSVERSE'] || '#64748b';
          const isSource = connectingSource === node.id;

          return (
            <div
              key={node.id}
              onClick={() => handleNodeClick(node.id)}
              style={{
                position: 'absolute',
                left: node.x,
                top: node.y,
                width: 120,
                height: 70,
                backgroundColor: 'var(--bg-panel)',
                border: `2px solid ${isSource ? 'var(--color-primary)' : color}`,
                borderLeft: `6px solid ${color}`,
                borderRadius: 'var(--radius-sm)',
                boxShadow: isSource ? '0 0 0 4px rgba(59, 130, 246, 0.3)' : 'var(--shadow-md)',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                color: 'var(--text-main)',
                fontSize: '0.8rem',
                zIndex: 10,
                cursor:
                  activeTool === 'select' ? 'move' : activeTool === 'connect' ? 'alias' : 'pointer',
                transition: 'all 0.1s',
                transform: isSource ? 'scale(1.05)' : 'scale(1)',
              }}
            >
              <span
                style={{
                  fontWeight: 'bold',
                  fontSize: '0.7rem',
                  color: color,
                  marginBottom: '4px',
                }}
              >
                {node.layer}
              </span>
              <span
                style={{
                  textAlign: 'center',
                  padding: '0 4px',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  maxWidth: '100%',
                }}
              >
                {node.label}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
