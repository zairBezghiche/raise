import { useRulesEngine } from '../../hooks/useRulesEngine';
import { invoke } from '@tauri-apps/api/core';

const SPACE = 'demo_space';
const DB = 'demo_db';

export default function ModelRulesDemo() {
  const { doc, handleChange, isCalculating, error } = useRulesEngine({
    space: SPACE,
    db: DB,
    collection: 'logical_functions',
    initialDoc: {
      name: 'COMPUTE_VELOCITY', // Nom incorrect (manque LF_)
      parent_pkg: 'Pkg_Navigation',
      description: 'Calcule la vitesse sol basée sur le GPS.',
      full_path: '',
      compliance: '',
    },
  });

  const initModel = async () => {
    try {
      await invoke('jsondb_init_model_rules', { space: SPACE, db: DB });
      alert("✅ Règles d'ingénierie chargées (Schéma LogicalFunction)");
      // Force refresh
      handleChange('name', doc.name);
    } catch (e) {
      console.error(e);
      alert('❌ Erreur: ' + e);
    }
  };

  // Helper pour la couleur du badge
  const isCompliant = doc.compliance?.includes('VALIDE');

  return (
    <div className="flex flex-col h-full bg-gray-50 p-6">
      <div className="flex justify-between items-end mb-6">
        <div>
          <h2 className="text-2xl font-bold text-slate-800">📐 Propriétés Élément</h2>
          <p className="text-slate-500">Validation des règles de modélisation Arcadia</p>
        </div>
        <button
          onClick={initModel}
          className="px-3 py-1 bg-slate-200 text-slate-700 rounded hover:bg-slate-300 text-xs font-bold uppercase tracking-wide"
        >
          Reset Rules
        </button>
      </div>

      <div className="bg-white rounded-lg shadow-sm border border-slate-200 overflow-hidden">
        {/* Header style "Capella" */}
        <div className="bg-slate-100 px-4 py-2 border-b border-slate-200 flex justify-between items-center">
          <span className="font-mono text-sm font-semibold text-slate-600">LogicalFunction</span>
          <div
            className={`text-xs px-2 py-0.5 rounded-full border ${
              isCompliant
                ? 'bg-green-100 text-green-700 border-green-200'
                : 'bg-red-100 text-red-700 border-red-200'
            }`}
          >
            {isCalculating ? 'Calcul...' : doc.compliance || 'Unknown'}
          </div>
        </div>

        <div className="p-6 grid grid-cols-1 gap-6">
          {/* CHAMPS ÉDITABLES */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-400 uppercase mb-1">
                Parent Package
              </label>
              <input
                type="text"
                value={doc.parent_pkg}
                onChange={(e) => handleChange('parent_pkg', e.target.value)}
                className="w-full p-2 border border-slate-300 rounded focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition"
              />
            </div>
            <div>
              <label className="block text-xs font-bold text-slate-400 uppercase mb-1">Name</label>
              <input
                type="text"
                value={doc.name}
                onChange={(e) => handleChange('name', e.target.value)}
                className={`w-full p-2 border rounded outline-none transition ${
                  !isCompliant && !isCalculating
                    ? 'border-red-300 bg-red-50 text-red-900'
                    : 'border-slate-300 focus:border-blue-500'
                }`}
              />
              {!isCompliant && !isCalculating && (
                <p className="text-xs text-red-500 mt-1">⚠️ Convention: LF_ + MAJUSCULES</p>
              )}
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-400 uppercase mb-1">
              Description
            </label>
            <textarea
              rows={3}
              value={doc.description}
              onChange={(e) => handleChange('description', e.target.value)}
              className="w-full p-2 border border-slate-300 rounded focus:border-blue-500 outline-none text-sm text-slate-600"
            />
          </div>

          {/* CHAMPS CALCULÉS (READ-ONLY) */}
          <div className="bg-slate-50 p-4 rounded border border-slate-200">
            <label className="block text-xs font-bold text-slate-400 uppercase mb-1">
              Computed Full Path (GenRules)
            </label>
            <code className="block w-full text-sm font-mono text-blue-700 break-all">
              {doc.full_path || '...'}
            </code>
          </div>

          {error && (
            <div className="text-red-600 text-sm bg-red-50 p-2 rounded border border-red-100">
              Erreur Système : {error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
