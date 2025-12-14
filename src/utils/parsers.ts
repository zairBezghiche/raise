/**
 * Extrait un message d'erreur lisible d'un objet erreur inconnu (Try/Catch).
 */
export function parseError(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  // Correction : Vérification et cast sécurisé sans 'any'
  if (typeof error === 'object' && error !== null && 'message' in error) {
    // On sait que 'message' existe, on le cast pour y accéder
    return String((error as Record<string, unknown>).message);
  }
  return 'Erreur inconnue';
}

/**
 * Extrait l'extension d'un nom de fichier.
 */
export function getFileExtension(filename: string): string {
  return filename.slice((Math.max(0, filename.lastIndexOf('.')) || Infinity) + 1);
}

/**
 * Tente de parser un JSON de manière sécurisée (retourne null si échec).
 */
export function safeJsonParse<T>(str: string): T | null {
  try {
    return JSON.parse(str);
  } catch {
    return null;
  }
}

/**
 * Extrait le nom d'un projet depuis son chemin de fichier.
 * ex: "/home/user/docs/MonProjet.aird" -> "MonProjet"
 */
export function getProjectNameFromPath(path: string): string {
  // Gestion Windows (\) et Unix (/)
  const cleanPath = path.replace(/\\/g, '/');
  const filename = cleanPath.split('/').pop() || '';
  return filename.split('.')[0];
}
