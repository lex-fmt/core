import { test, _electron as electron } from '@playwright/test';
import * as path from 'path';
import { openFixture } from './helpers';

test.describe('Debug Clicks', () => {
  test('should log token info on clicks', async () => {
    const appPath = path.join(process.cwd(), 'release/mac-arm64/Lex Editor.app/Contents/MacOS/Lex Editor');
    
    const app = await electron.launch({
      executablePath: appPath,
      args: [path.join(process.cwd(), 'specs/v1/benchmark/30-a-place-for-ideas.lex')],
      env: {
        ...process.env,
        LEX_TEST_FIXTURES: path.join(process.cwd(), 'tests/fixtures'),
      },
    });

    const page = await app.firstWindow();
    page.on('console', msg => console.log(`[Browser Console] ${msg.type()}: ${msg.text()}`));
    await page.waitForLoadState('domcontentloaded');

    await openFixture(page, 'benchmark.lex');

    // Wait for editor to be ready
    await page.waitForSelector('.monaco-editor');
    await page.waitForTimeout(2000); // Give LSP time to initialize

    const clicks = [
      { line: 3, column: 5, desc: 'Session Title (The)' },
      { line: 3, column: 1, desc: 'Seq Marker (1)' },
      { line: 5, column: 9, desc: 'Paragraph (know)' },
      { line: 71, column: 17, desc: 'Verbatim Label (python)' },
      { line: 64, column: 20, desc: 'Verbatim Content (parse)' },
    ];

    for (const click of clicks) {
      console.log(`\n--- Clicking: ${click.desc} ---`);
      
      // Simulate click in Monaco Editor
      // We use executeJavaScript to access the Monaco editor instance directly
      await page.evaluate(({ line, column }) => {
        const editor = (window as any).editor;
        if (!editor) {
          console.warn('Editor instance not available');
          return;
        }
        const position = { lineNumber: line, column: column };
        editor.setPosition(position);
        editor.revealPosition(position);
        
        // Trigger the mouse down handler we added
        // We need to simulate the event object structure expected by our handler
        const model = typeof editor.getModel === 'function' ? editor.getModel() : null;
        if (!model) {
          console.warn('Editor model not ready');
          return;
        }
        const word = model.getWordAtPosition(position);
        const offset = model.getOffsetAt(position);
        
        // Manually trigger the logic since we can't easily simulate a real mouse event with accurate target.position in Playwright
        // But wait, we added editor.onMouseDown. 
        // We can dispatch a synthetic event if we can target the right element, but Monaco's DOM is complex.
        // Easier to just call the logging logic directly or trigger the event via Monaco API if possible.
        // Actually, let's just use the same logic we added to Editor.tsx inside evaluate
        
        console.log('--- Click Debug (Simulated) ---');
        console.log('Position:', position);
        console.log('Word:', word);
        console.log('Offset:', offset);
        
        // @ts-ignore
        const tokens = (window as any).monaco.editor.tokenize(model.getValue(), model.getLanguageId());
        if (tokens && tokens[position.lineNumber - 1]) {
            const lineTokens = tokens[position.lineNumber - 1];
            // Find the token that covers the column. 
            // Tokens are just { offset, type, language }. 
            // The token covers from its offset up to the next token's offset.
            // We need to find the last token with offset <= column - 1
            let token = null;
            for (let i = 0; i < lineTokens.length; i++) {
                if (lineTokens[i].offset <= position.column - 1) {
                    token = lineTokens[i];
                } else {
                    break;
                }
            }
            
            console.log('Monarch Token (Line):', token ? JSON.stringify(token) : 'null');
            console.log('All Line Tokens:', JSON.stringify(lineTokens));
        }
        
      }, click);

      // Wait a bit to capture logs
      await page.waitForTimeout(500);
    }

    await app.close();
  });
});
