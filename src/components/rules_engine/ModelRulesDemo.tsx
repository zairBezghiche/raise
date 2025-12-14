import { useRulesEngine } from '@/hooks/useRulesEngine';

// Interface locale pour typer les données de cette démo
interface ModelRuleData {
  name?: string;
  parent_pkg?: string;
  description?: string;
  compliance?: string; // Calculé par le moteur
  full_path?: string; // Calculé par le moteur
  [key: string]: unknown;
}

export default function ModelRulesDemo() {
  const { doc, handleChange, isCalculating } = useRulesEngine({
    space: 'demo',
    db: 'architecture',
    collection: 'components',
    initialDoc: {
      name: 'UserSystem',
      parent_pkg: 'com.company.core',
      description: '',
      // compliance et full_path sont calculés
    },
  });

  // Cast vers notre interface
  const data = doc as ModelRuleData;

  // Calcul visuel basé sur le retour du moteur
  const isCompliant = typeof data.compliance === 'string' && data.compliance.includes('VALIDE');

  return (
    <div className="p-4 bg-white rounded shadow-sm border border-gray-200 mt-6">
      <h3 className="text-lg font-bold text-gray-700 mb-4">
        🏗️ Démo 2 : Validation Architecture (Compliance)
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Formulaire */}
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-600">Package Parent</label>
            <input
              type="text"
              placeholder="ex: com.company.module"
              className="mt-1 block w-full border border-gray-300 rounded px-3 py-2"
              value={data.parent_pkg || ''}
              onChange={(e) => handleChange('parent_pkg', e.target.value)}
            />
            <p className="text-xs text-gray-400 mt-1">Doit commencer par 'com.company'</p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600">Nom Composant</label>
            <input
              type="text"
              className="mt-1 block w-full border border-gray-300 rounded px-3 py-2"
              value={data.name || ''}
              onChange={(e) => handleChange('name', e.target.value)}
            />
            <p className="text-xs text-gray-400 mt-1">Doit être en PascalCase</p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600">Description</label>
            <textarea
              className="mt-1 block w-full border border-gray-300 rounded px-3 py-2"
              rows={3}
              value={data.description || ''}
              onChange={(e) => handleChange('description', e.target.value)}
            />
          </div>
        </div>

        {/* Résultat Live */}
        <div className="bg-gray-50 p-4 rounded border border-gray-200 flex flex-col justify-between">
          <div>
            <h4 className="font-semibold text-gray-500 uppercase text-xs mb-4">Audit Temps Réel</h4>

            <div
              className={`p-3 rounded border mb-4 text-center font-bold ${
                isCompliant
                  ? 'bg-green-100 border-green-300 text-green-700'
                  : 'bg-red-100 border-red-300 text-red-700'
              }`}
            >
              {isCalculating ? 'Calcul...' : data.compliance || 'Unknown'}
            </div>

            <div className="space-y-2 text-sm">
              <div className="flex flex-col">
                <span className="text-gray-500 text-xs">Chemin complet généré :</span>
                <code className="font-mono text-gray-800 bg-white p-1 border rounded mt-1">
                  {data.full_path || '...'}
                </code>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
