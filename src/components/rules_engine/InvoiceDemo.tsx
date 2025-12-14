import { useRulesEngine } from '../../hooks/useRulesEngine';
import { invoke } from '@tauri-apps/api/core';

const SPACE = 'demo_space';
const DB = 'demo_db';

export default function InvoiceDemo() {
  const { doc, handleChange, isCalculating, error } = useRulesEngine({
    space: SPACE,
    db: DB,
    collection: 'invoices',
    initialDoc: {
      user_id: 'u_dev',
      days: 10,
      created_at: new Date().toISOString().split('T')[0],
      total: 0,
      due_at: '',
      ref: '',
    },
  });

  const initDemo = async () => {
    try {
      // Nous allons créer cette commande Rust juste après pour faciliter la vie
      await invoke('jsondb_init_demo_rules', { space: SPACE, db: DB });
      alert('✅ Environnement de démo prêt (Schémas + User + Collection)');
      // On force un rafraichissement du calcul
      handleChange('days', doc.days);
    } catch (e) {
      console.error(e);
      alert('❌ Erreur init: ' + e);
    }
  };

  return (
    <div className="p-6 max-w-2xl mx-auto bg-white rounded-xl shadow-md border border-gray-200 space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-xl font-bold text-gray-800">🧾 Démo Facture (GenRules)</h2>
        <button
          onClick={initDemo}
          className="px-4 py-2 bg-blue-100 text-blue-700 rounded hover:bg-blue-200 text-sm font-semibold"
        >
          🛠️ Setup Démo
        </button>
      </div>

      <p className="text-sm text-gray-500">
        Le backend Rust calcule le total (Jours × TJM), la date (+30j) et la référence en temps
        réel.
      </p>

      <div className="grid grid-cols-1 gap-4">
        {/* INPUTS */}
        <div>
          <label className="block text-sm font-medium text-gray-700">
            Utilisateur ID (Lookup DB)
          </label>
          <input
            type="text"
            value={doc.user_id}
            onChange={(e) => handleChange('user_id', e.target.value)}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 p-2 border"
          />
          <span className="text-xs text-gray-400">Essayez "u_dev" (TJM: 500€)</span>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">Jours travaillés</label>
          <input
            type="number"
            value={doc.days}
            onChange={(e) => handleChange('days', parseFloat(e.target.value) || 0)}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 p-2 border"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">Date Création</label>
          <input
            type="date"
            value={doc.created_at?.split('T')[0]}
            onChange={(e) => handleChange('created_at', new Date(e.target.value).toISOString())}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 p-2 border"
          />
        </div>

        <hr className="my-2" />

        {/* RÉSULTATS CALCULÉS */}
        <div className="bg-slate-50 p-4 rounded-lg border border-slate-200">
          <div className="flex justify-between items-center mb-2">
            <span className="font-semibold text-gray-700">Total HT (Calculé):</span>
            <span
              className={`text-2xl font-bold ${
                isCalculating ? 'text-orange-400' : 'text-green-600'
              }`}
            >
              {isCalculating ? '...' : `${doc.total} €`}
            </span>
          </div>

          <div className="text-sm text-gray-600 flex justify-between">
            <strong>Échéance (+30j):</strong>
            <span>{doc.due_at ? new Date(doc.due_at).toLocaleDateString() : '-'}</span>
          </div>

          <div className="text-sm text-gray-600 flex justify-between mt-1">
            <strong>Référence Auto:</strong>
            <code className="bg-gray-200 px-2 rounded text-gray-800">{doc.ref || '-'}</code>
          </div>
        </div>

        {error && (
          <div className="p-3 bg-red-50 text-red-700 rounded border border-red-200 text-sm">
            ⚠️ {error}
          </div>
        )}
      </div>
    </div>
  );
}
