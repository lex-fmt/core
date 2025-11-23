import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runTests } from '@vscode/test-electron';

async function main() {
  const currentDir = fileURLToPath(new URL('.', import.meta.url));
  const extensionDevelopmentPath = path.resolve(currentDir, '..', '..');
  const extensionTestsPath = path.resolve(currentDir, 'integration/index.js');
  const workspacePath = path.resolve(
    extensionDevelopmentPath,
    'test/fixtures/sample-workspace'
  );

  try {
    await runTests({
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: [workspacePath],
      extensionTestsEnv: {
        ...process.env,
        LEX_VSCODE_SKIP_SERVER: '1'
      }
    });
  } catch (error) {
    console.error('Failed to run VS Code extension tests');
    console.error(error);
    process.exit(1);
  }
}

main();
