import type { Page } from '@playwright/test';

export async function openFixture(page: Page, fixtureName: string) {
  await page.waitForFunction(() => Boolean((window as any).lexTest), null, { timeout: 5000 });
  return await page.evaluate(async (name) => {
    if (!window.lexTest) {
      throw new Error('lexTest helpers not available');
    }
    return await window.lexTest.openFixture(name);
  }, fixtureName);
}
