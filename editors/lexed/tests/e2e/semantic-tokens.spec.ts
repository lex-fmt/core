import { test, expect, _electron as electron } from '@playwright/test';
import { openFixture } from './helpers';

test.describe('Semantic Tokens', () => {
  test('should request and apply semantic tokens from LSP', async () => {
    test.setTimeout(60000);

    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();

    // Capture console logs
    const logs: string[] = [];
    window.on('console', msg => {
      const text = `[${msg.type()}] ${msg.text()}`;
      logs.push(text);
    });

    await window.waitForLoadState('domcontentloaded');
    await openFixture(window, 'semantic-basic.lex');
    await expect(window.locator('.monaco-editor').first()).toBeVisible();

    // Wait for semantic tokens to be received
    await window.waitForTimeout(5000);

    // Check if semantic tokens provider was triggered and received tokens
    const providerTriggered = logs.some(log => log.includes('[SemanticTokens] Provider triggered'));
    const tokensReceived = logs.some(log => log.includes('[SemanticTokens] Received tokens:'));

    // Extract the number of tokens received
    const tokenCountLog = logs.find(log => log.includes('[SemanticTokens] Received tokens:'));
    const tokenCount = tokenCountLog ? parseInt(tokenCountLog.replace(/[^0-9]/g, '') || '0', 10) : 0;

    console.log('Provider triggered:', providerTriggered);
    console.log('Tokens received:', tokensReceived);
    console.log('Token count:', tokenCount);

    await electronApp.close();

    // Verify semantic tokens are working
    expect(providerTriggered).toBe(true);
    expect(tokensReceived).toBe(true);
    expect(tokenCount).toBeGreaterThan(0);
  });
});
