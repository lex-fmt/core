import { rmSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import sharp from 'sharp';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.join(__dirname, '..');
const workspaceRoot = path.join(projectRoot, '..', '..');
const outputDir = path.join(projectRoot, 'build', 'icons');
const tmpDir = path.join(projectRoot, 'build');
const basePng = path.join(tmpDir, 'icon-1024.png');
const logoPath = path.join(workspaceRoot, 'assets', 'logo.svg');

rmSync(outputDir, { recursive: true, force: true });
mkdirSync(outputDir, { recursive: true });
mkdirSync(tmpDir, { recursive: true });

await sharp(logoPath)
  .resize(1024, 1024, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
  .png()
  .toFile(basePng);

const npxCmd = process.platform === 'win32' ? 'npx.cmd' : 'npx';
const result = spawnSync(npxCmd, [
  'electron-icon-builder',
  '--input', basePng,
  '--output', outputDir,
  '--flatten',
], {
  cwd: projectRoot,
  stdio: 'inherit',
});

if (result.status !== 0) {
  throw new Error(`electron-icon-builder failed with code ${result.status}`);
}
