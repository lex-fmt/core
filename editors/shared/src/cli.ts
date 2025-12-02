import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, writeFileSync, unlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { FORMAT_EXTENSIONS } from './constants.js';

export interface ConvertOptions {
  cliBinaryPath: string;
  fromFormat: 'lex' | 'markdown';
  toFormat: 'lex' | 'markdown' | 'html';
  targetLanguageId: string;
}

export async function convertDocument(
  content: string,
  options: ConvertOptions
): Promise<string> {
  const { cliBinaryPath, fromFormat, toFormat } = options;

  if (!existsSync(cliBinaryPath)) {
    throw new Error(
      `Lex CLI binary not found at ${cliBinaryPath}. ` +
        'Configure lex.cliBinaryPath or ensure the bundled binary is available.'
    );
  }

  // Create a temporary file with the content
  const tmpDir = mkdtempSync(join(tmpdir(), 'lex-vscode-'));
  const inputExt = FORMAT_EXTENSIONS[fromFormat] || '.txt';
  const inputPath = join(tmpDir, `input${inputExt}`);

  try {
    writeFileSync(inputPath, content, 'utf-8');

    return await new Promise((resolve, reject) => {
      const args = ['convert', '--to', toFormat, inputPath];
      const proc = spawn(cliBinaryPath, args, {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      let stdout = '';
      let stderr = '';

      proc.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      proc.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      proc.on('error', (err: Error) => {
        reject(new Error(`Failed to spawn lex CLI: ${err.message}`));
      });

      proc.on('close', (code: number | null) => {
        if (code !== 0) {
          reject(
            new Error(`lex convert failed (exit ${code}): ${stderr || 'unknown error'}`)
          );
          return;
        }
        resolve(stdout);
      });
    });
  } finally {
    // Cleanup temp files
    try {
      unlinkSync(inputPath);
      rmSync(tmpDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

/**
 * Convert a Lex document to PDF and write it to the specified output file.
 * PDF requires file-based output since it's binary.
 */
export async function convertToPdfFile(
  content: string,
  cliBinaryPath: string,
  outputPath: string
): Promise<void> {
  if (!existsSync(cliBinaryPath)) {
    throw new Error(
      `Lex CLI binary not found at ${cliBinaryPath}. ` +
        'Configure lex.cliBinaryPath or ensure the bundled binary is available.'
    );
  }

  // Create a temporary file with the content
  const tmpDir = mkdtempSync(join(tmpdir(), 'lex-vscode-'));
  const inputPath = join(tmpDir, 'input.lex');

  try {
    writeFileSync(inputPath, content, 'utf-8');

    await new Promise<void>((resolve, reject) => {
      const args = ['convert', '--to', 'pdf', '--output', outputPath, inputPath];
      const proc = spawn(cliBinaryPath, args, {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      let stderr = '';

      proc.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      proc.on('error', (err: Error) => {
        reject(new Error(`Failed to spawn lex CLI: ${err.message}`));
      });

      proc.on('close', (code: number | null) => {
        if (code !== 0) {
          reject(
            new Error(`lex convert failed (exit ${code}): ${stderr || 'unknown error'}`)
          );
          return;
        }
        resolve();
      });
    });
  } finally {
    // Cleanup temp files
    try {
      unlinkSync(inputPath);
      rmSync(tmpDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

/**
 * Convert Lex content to HTML. Used by both export command and live preview.
 */
export async function convertToHtml(
  content: string,
  cliBinaryPath: string
): Promise<string> {
  return convertDocument(content, {
    cliBinaryPath,
    fromFormat: 'lex',
    toFormat: 'html',
    targetLanguageId: 'html'
  });
}

export interface ConvertFileOptions {
  cliBinaryPath: string;
  sourcePath: string;
  outputPath: string;
  toFormat: 'lex' | 'markdown' | 'html' | 'pdf';
}

export async function convertFile(options: ConvertFileOptions): Promise<void> {
  const { cliBinaryPath, sourcePath, outputPath, toFormat } = options;

  if (!existsSync(cliBinaryPath)) {
    throw new Error(
      `Lex CLI binary not found at ${cliBinaryPath}. ` +
        'Configure lex.cliBinaryPath or ensure the bundled binary is available.'
    );
  }

  return new Promise((resolve, reject) => {
    const args = ['convert', sourcePath, '--to', toFormat, '-o', outputPath];
    const proc = spawn(cliBinaryPath, args, {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let stderr = '';
    proc.stderr.on('data', (data: Buffer) => {
      stderr += data.toString();
    });

    proc.on('error', (err: Error) => {
      reject(new Error(`Failed to spawn lex CLI: ${err.message}`));
    });

    proc.on('close', (code: number | null) => {
      if (code !== 0) {
        reject(
          new Error(`lex convert failed (exit ${code}): ${stderr || 'unknown error'}`)
        );
        return;
      }
      resolve();
    });
  });
}
