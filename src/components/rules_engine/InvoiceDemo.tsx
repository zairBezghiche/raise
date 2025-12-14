import { useRulesEngine } from '@/hooks/useRulesEngine';

// Interface locale pour typer les données de cette démo
interface InvoiceData {
  user_id?: string;
  days?: number | string; // Accepte string pour l'input
  created_at?: string;
  due_at?: string;
  ref?: string;
  amount?: number;
  // Permet d'autres champs dynamiques
  [key: string]: unknown;
}

export default function InvoiceDemo() {
  const { doc, handleChange, isCalculating, error } = useRulesEngine({
    space: 'demo',
    db: 'finance',
    collection: 'invoices',
    initialDoc: {
      user_id: 'USR_123',
      days: 30, // Délai de paiement
      created_at: new Date().toISOString(),
      // ref, amount, due_at seront calculés par le moteur
    },
  });

  // Cast du document générique vers notre structure attendue
  const data = doc as InvoiceData;

  return (
    <div className="p-4 bg-white rounded shadow-sm border border-gray-200">
      <h3 className="text-lg font-bold text-gray-700 mb-4">
        🧾 Démo 1 : Facturation (Calculs de Dates)
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Formulaire */}
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-600">ID Utilisateur</label>
            <input
              type="text"
              className="mt-1 block w-full border border-gray-300 rounded px-3 py-2"
              value={data.user_id || ''}
              onChange={(e) => handleChange('user_id', e.target.value)}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600">Délai (jours)</label>
            <input
              type="number"
              className="mt-1 block w-full border border-gray-300 rounded px-3 py-2"
              value={data.days || 0}
              onChange={(e) => handleChange('days', parseInt(e.target.value) || 0)}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600">Date Création</label>
            <input
              type="date"
              className="mt-1 block w-full border border-gray-300 rounded px-3 py-2"
              // On sécurise l'accès et le split
              value={typeof data.created_at === 'string' ? data.created_at.split('T')[0] : ''}
              onChange={(e) => handleChange('created_at', new Date(e.target.value).toISOString())}
            />
          </div>
        </div>

        {/* Résultat Live */}
        <div className="bg-gray-50 p-4 rounded border border-gray-200 flex flex-col justify-between">
          <div>
            <h4 className="font-semibold text-gray-500 uppercase text-xs mb-4">
              Résultat Moteur (Rust)
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Date Échéance :</span>
                <span className="font-mono font-bold text-blue-600">
                  {data.due_at ? new Date(data.due_at).toLocaleDateString() : '-'}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span>Référence :</span>
                <code className="bg-gray-200 px-2 rounded text-gray-800">{data.ref || '-'}</code>
              </div>
            </div>
          </div>

          {error && <div className="mt-4 text-red-500 text-xs">{error}</div>}

          <div className="mt-4 text-xs text-gray-400 text-right">
            {isCalculating ? '⚡ Calcul...' : '✅ Synchronisé'}
          </div>
        </div>
      </div>
    </div>
  );
}
