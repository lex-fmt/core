/// <reference lib="webworker" />

// Load the library globally
importScripts('/libs/typo.js');

declare class Typo {
    constructor(lang: string, aff: string, dic: string);
    check(word: string): boolean;
    suggest(word: string): string[];
}

let typo: Typo | null = null;

const log = (msg: string, data?: any) => {
    self.postMessage({ type: 'debug', payload: { msg, data } });
};

self.onmessage = (e: MessageEvent) => {
  const { type, payload } = e.data;
  // log(`Worker received message: ${type}`);

  switch (type) {
    case 'init':
      try {
        const { lang, aff, dic } = payload;
        log(`Initializing Typo for ${lang}`);
        
        // Typo is now a global class
        typo = new Typo(lang, aff, dic);
        
        log('Typo initialized');
        self.postMessage({ type: 'init_complete', payload: { success: true } });
      } catch (error) {
        log('Failed to initialize Typo', String(error));
        self.postMessage({ type: 'init_complete', payload: { success: false, error: String(error) } });
      }
      break;

    case 'check':
      if (!typo) {
          log('Check requested but Typo not initialized');
          return;
      }
      const { words, id } = payload;
      const misspelled: { word: string, index: number }[] = [];
      
      words.forEach((item: { word: string, index: number }) => {
        if (!typo!.check(item.word)) {
          misspelled.push(item);
        }
      });
      
      self.postMessage({ type: 'check_result', payload: { id, misspelled } });
      break;

    case 'suggest':
      if (!typo) return;
      const { word, id: suggestId } = payload;
      const suggestions = typo.suggest(word).slice(0, 4);
      self.postMessage({ type: 'suggest_result', payload: { id: suggestId, suggestions } });
      break;
  }
};
