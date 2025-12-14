import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface RulesEngineOptions {
  space: string;
  db: string;
  collection: string;
  // Correction : Remplacement de any par unknown pour le typage strict
  initialDoc: Record<string, unknown>;
  debounceMs?: number;
}

export function useRulesEngine({
  space,
  db,
  collection,
  initialDoc,
  debounceMs = 500,
}: RulesEngineOptions) {
  // Correction : State typé avec Record<string, unknown>
  const [doc, setDoc] = useState<Record<string, unknown>>(initialDoc);
  const [isCalculating, setIsCalculating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Ref pour éviter les boucles infinies si le backend renvoie la même chose
  const lastEvaluatedDoc = useRef<string>(JSON.stringify(initialDoc));

  useEffect(() => {
    const currentDocStr = JSON.stringify(doc);
    if (currentDocStr === lastEvaluatedDoc.current) return;

    const timer = setTimeout(async () => {
      setIsCalculating(true);
      setError(null);

      try {
        console.log('⚡ [Rules] Evaluation en cours...');
        // Correction : invoke retourne un Record<string, unknown>
        const updatedDoc = await invoke<Record<string, unknown>>('jsondb_evaluate_draft', {
          space,
          db,
          collection,
          doc,
        });

        const updatedDocStr = JSON.stringify(updatedDoc);
        if (updatedDocStr !== currentDocStr) {
          lastEvaluatedDoc.current = updatedDocStr;
          setDoc(updatedDoc);
        } else {
          lastEvaluatedDoc.current = currentDocStr;
        }
      } catch (err: unknown) {
        // Correction : Typage explicite de l'erreur
        console.error('❌ [Rules] Erreur:', err);
        setError(String(err));
      } finally {
        setIsCalculating(false);
      }
    }, debounceMs);

    return () => clearTimeout(timer);
  }, [doc, space, db, collection, debounceMs]);

  // Correction : La valeur peut être inconnue (string, number, boolean...)
  const handleChange = useCallback((field: string, value: unknown) => {
    setDoc((prev) => ({ ...prev, [field]: value }));
  }, []);

  return { doc, setDoc, handleChange, isCalculating, error };
}
