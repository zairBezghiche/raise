interface ExecutionMonitorProps {
  logs: string[];
  status: string;
  onStart: () => void;
  isRunning: boolean;
}

export function ExecutionMonitor({ logs, status, onStart, isRunning }: ExecutionMonitorProps) {
  return (
    <div
      style={{
        height: '200px',
        backgroundColor: 'var(--bg-panel)',
        borderTop: '1px solid var(--border-color)',
        display: 'flex',
        flexDirection: 'column',
        zIndex: 20,
      }}
    >
      <header
        style={{
          padding: 'var(--spacing-2) var(--spacing-4)',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          backgroundColor: 'var(--bg-app)',
        }}
      >
        <span style={{ fontSize: 'var(--font-size-xs)', fontWeight: 'bold' }}>
          🖥️ Console d'exécution ({status})
        </span>
        <button
          onClick={onStart}
          disabled={isRunning || status === 'PAUSED'}
          style={{
            padding: 'var(--spacing-2) var(--spacing-4)',
            backgroundColor: isRunning ? 'var(--color-gray-400)' : 'var(--color-success)',
            color: '#fff',
            border: 'none',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--font-size-xs)',
            cursor: isRunning ? 'wait' : 'pointer',
            fontWeight: 'bold',
          }}
        >
          {isRunning ? 'En cours...' : '▶ Lancer le Workflow'}
        </button>
      </header>

      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: 'var(--spacing-2)',
          fontFamily: 'var(--font-family-mono)',
          fontSize: 'var(--font-size-xs)',
          backgroundColor: '#1e1e1e', // Fond terminal
          color: '#d4d4d4',
          display: 'flex',
          flexDirection: 'column-reverse', // Auto-scroll vers le bas (les derniers logs en bas)
        }}
      >
        {logs.length === 0 && <span style={{ opacity: 0.5 }}>En attente...</span>}

        {/* On map les logs reçus de Rust */}
        {[...logs].reverse().map((log, i) => (
          <div
            key={i}
            style={{
              borderLeft: '2px solid var(--color-primary)',
              paddingLeft: '8px',
              marginBottom: '4px',
            }}
          >
            {log}
          </div>
        ))}
      </div>
    </div>
  );
}
