import { rmSync, mkdirSync, renameSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';
import iconGen from 'icon-gen';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.join(__dirname, '..');
const workspaceRoot = path.join(projectRoot, '..', '..');
const outputDir = path.join(projectRoot, 'build', 'icons');
const iconsDir = path.join(outputDir, 'icons');
const pngOutputDir = iconsDir;
const macOutputDir = iconsDir;
const winOutputDir = iconsDir;
const tmpDir = path.join(projectRoot, 'build');
const basePng = path.join(tmpDir, 'icon-1024.png');
const pngSizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024];
const logoPath = path.join(workspaceRoot, 'assets', 'logo.svg');

rmSync(outputDir, { recursive: true, force: true });
mkdirSync(iconsDir, { recursive: true });
mkdirSync(tmpDir, { recursive: true });

await sharp(logoPath)
  .resize(1024, 1024, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
  .png()
  .toFile(basePng);

for (const size of pngSizes) {
  const target = path.join(pngOutputDir, `${size}.png`);
  await sharp(basePng)
    .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(target);
  console.log(`Created ${target}`);
}

await iconGen(pngOutputDir, macOutputDir, {
  icns: { name: 'icon' },
  report: true,
});

await iconGen(pngOutputDir, winOutputDir, {
  ico: { name: 'icon' },
  report: true,
});

console.log('Renaming PNGs to Electron Format');
for (const size of pngSizes) {
  const startName = `${size}.png`;
  const endName = `${size}x${size}.png`;
  const from = path.join(pngOutputDir, startName);
  const to = path.join(pngOutputDir, endName);
  renameSync(from, to);
  console.log(`Renamed ${startName} to ${endName}`);
}

console.log('\n ALL DONE');
