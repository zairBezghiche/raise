import { useEffect, useState } from 'react';

export function ThemeToggle() {
  // CORRECTION : On initialise l'état directement avec la valeur du localStorage
  // Cela évite de devoir appeler setTheme dans un useEffect au montage (ce qui causait l'erreur)
  const [theme, setTheme] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('app-theme') || 'light';
    }
    return 'light';
  });

  // On applique le thème au DOM à chaque changement de la variable 'theme'
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('app-theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === 'light' ? 'dark' : 'light'));
  };

  return (
    <button
      type="button"
      onClick={toggleTheme}
      style={{
        background: 'transparent',
        border: '1px solid var(--border-color, #ccc)',
        borderRadius: '20px',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        padding: '6px 12px',
        color: 'var(--text-main, inherit)',
        fontSize: '0.9rem',
        transition: 'all 0.2s',
      }}
      aria-label={theme === 'light' ? 'Passer en mode sombre' : 'Passer en mode clair'}
    >
      <span aria-hidden="true" style={{ fontSize: '1.2em', lineHeight: 1 }}>
        {theme === 'light' ? '🌙' : '☀️'}
      </span>
      <span style={{ marginLeft: 8, fontWeight: 500 }}>
        {theme === 'light' ? 'Mode Sombre' : 'Mode Clair'}
      </span>
    </button>
  );
}
