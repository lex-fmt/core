import { lspClient } from './client';

let initializePromise: Promise<void> | null = null;

export function ensureLspInitialized() {
  if (!initializePromise) {
    initializePromise = lspClient.start().catch(error => {
      console.error('LSP initialization failed', error);
      throw error;
    });
  }
  return initializePromise;
}
