// FICHIER : src/components/demo/BlockchainToast.tsx

import { useEffect, useState } from 'react';

export const BlockchainToast = ({ trigger }: { trigger: boolean }) => {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (trigger) {
      setVisible(true);
      // La notification disparaît toute seule après 8 secondes
      const timer = setTimeout(() => setVisible(false), 8000);
      return () => clearTimeout(timer);
    }
  }, [trigger]);

  if (!visible) return null;

  return (
    <>
      <style>
        {`
          @keyframes slideUp {
            from { transform: translateY(100%); opacity: 0; }
            to { transform: translateY(0); opacity: 1; }
          }
          @keyframes pulse-green {
            0% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.4); }
            70% { box-shadow: 0 0 0 10px rgba(16, 185, 129, 0); }
            100% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
          }
        `}
      </style>
      <div
        style={{
          position: 'fixed',
          bottom: '30px',
          right: '30px',
          background: '#064e3b', // Vert très sombre (Défense/Sécurité)
          border: '1px solid #10b981', // Bordure Vert Matrix
          color: '#ecfdf5',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 0 15px rgba(16, 185, 129, 0.2)',
          fontFamily: "'Courier New', monospace", // Font "Code"
          zIndex: 9999,
          display: 'flex',
          alignItems: 'center',
          gap: '15px',
          minWidth: '400px',
          animation: 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1), pulse-green 2s infinite',
        }}
      >
        {/* Icône de chaîne / Blockchain */}
        <div
          style={{
            fontSize: '28px',
            background: 'rgba(16, 185, 129, 0.2)',
            borderRadius: '50%',
            width: '50px',
            height: '50px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          🔗
        </div>

        <div>
          <div
            style={{
              textTransform: 'uppercase',
              fontWeight: 'bold',
              letterSpacing: '1px',
              fontSize: '0.9em',
              marginBottom: '4px',
              color: '#34d399',
            }}
          >
            Blockchain Consensus Reached
          </div>

          <div style={{ fontSize: '0.85em', opacity: 0.8, marginBottom: '2px' }}>
            Decision Hash:{' '}
            <span style={{ fontFamily: 'monospace', color: '#6ee7b7' }}>0x8f4c...e2a4</span>
          </div>

          <div
            style={{
              fontSize: '0.75em',
              background: '#065f46',
              padding: '2px 6px',
              borderRadius: '4px',
              display: 'inline-block',
              marginTop: '4px',
            }}
          >
            ✓ Anchored on Hyperledger Fabric
          </div>
        </div>
      </div>
    </>
  );
};
