import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface RulesEngineOptions {
  space: string;
  db: string;
  collection: string;
  initialDoc: Record<string, any>;
  debounceMs?: number;
}

export function useRulesEngine({
  space,
  db,
  collection,
  initialDoc,
  debounceMs = 500,
}: RulesEngineOptions) {
  const [doc, setDoc] = useState<Record<string, any>>(initialDoc);
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
        const updatedDoc = await invoke<Record<string, any>>('jsondb_evaluate_draft', {
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
      } catch (err) {
        console.error('❌ [Rules] Erreur:', err);
        setError(String(err));
      } finally {
        setIsCalculating(false);
      }
    }, debounceMs);

    return () => clearTimeout(timer);
  }, [doc, space, db, collection, debounceMs]);

  const handleChange = useCallback((field: string, value: any) => {
    setDoc((prev) => ({ ...prev, [field]: value }));
  }, []);

  return { doc, setDoc, handleChange, isCalculating, error };
}
