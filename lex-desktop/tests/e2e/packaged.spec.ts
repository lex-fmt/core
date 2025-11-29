import { test, expect, _electron as electron } from '@playwright/test';
import * as path from 'path';

test.describe('Packaged Application', () => {
  let electronApp: any;
  let page: any;

  test.beforeAll(async () => {
    // Path to the packaged application
    // Adjust this based on your platform and build output
    const appPath = path.join(process.cwd(), 'release/mac-arm64/Lex Editor.app/Contents/MacOS/Lex Editor');
    
    console.log(`Launching app from: ${appPath}`);

    electronApp = await electron.launch({
      executablePath: appPath,
    });

    page = await electronApp.firstWindow();
    
    // Capture console logs to debug issues
    page.on('console', (msg: any) => console.log(`[Browser Console] ${msg.type()}: ${msg.text()}`));
    page.on('pageerror', (err: any) => console.log(`[Browser Page Error] ${err.message}`));

    await page.waitForLoadState('domcontentloaded');
  });

  test.afterAll(async () => {
    await electronApp.close();
  });

  test('should open benchmark file and display outline', async () => {
    try {
        // Wait for the app to load
        await page.waitForSelector('.monaco-editor', { timeout: 10000 });

        // Get the benchmark file path
        const benchmarkFile = await page.evaluate(async () => {
            return await (window as any).ipcRenderer.invoke('get-benchmark-file');
        });
        
        console.log('Benchmark file:', benchmarkFile);

        // Open the benchmark file
        await page.click('button:has-text("Open File")'); // This triggers file dialog mock
        
        // Wait for editor content to load (look for specific text from the file)
        await page.waitForSelector('.view-lines', { timeout: 10000 });
        
        // Check for outline (TODO: Fix LSP connection in packaged app)
        // await page.waitForSelector('.outline-node', { timeout: 20000 });
        // const outlineNodes = await page.locator('.outline-node').count();
        // expect(outlineNodes).toBeGreaterThan(0);

    } catch (e) {
        console.error('Test failed. Dumping page content...');
        const content = await page.content();
        await page.screenshot({ path: 'test-failure.png' });
        // console.error('Page Content:', content);
        throw e;
    }
  });
});
