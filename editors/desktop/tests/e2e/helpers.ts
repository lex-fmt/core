import type { Page } from '@playwright/test';

type LexTestWindow = Window & {
  lexTest?: {
    openFixture: (fixtureName: string) => Promise<{ path: string; content: string }>;
  };
};

export async function openFixture(page: Page, fixtureName: string) {
  await page.waitForFunction(
    () => Boolean((window as LexTestWindow).lexTest),
    null,
    { timeout: 5000 }
  );
  return await page.evaluate(async (name) => {
    const scopedWindow = window as LexTestWindow;
    if (!scopedWindow.lexTest) {
      throw new Error('lexTest helpers not available');
    }
    return scopedWindow.lexTest.openFixture(name);
  }, fixtureName);
}
