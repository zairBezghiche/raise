import { useState } from 'react';
import { BlockchainToast } from './BlockchainToast';

export default function BlockchainView() {
  const [showToast, setShowToast] = useState(false);

  const handleAnchor = () => {
    setShowToast(true);
    setTimeout(() => setShowToast(false), 3000);
  };

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
        onClick={handleAnchor}
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

      {/* Le Toast est maintenant géré localement par la vue */}
      <BlockchainToast trigger={showToast} />
    </div>
  );
}
