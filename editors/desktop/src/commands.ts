import * as monaco from 'monaco-editor';
import { commands } from '@lex/shared';
import { MonacoEditorAdapter } from './adapter';

// In a real desktop app, we'd use Electron's dialog to pick files
// For now, we'll assume the main process exposes a way to pick files via IPC
// or we can just mock it for this refactor if the IPC isn't ready.
// Based on Editor.tsx, we have window.ipcRenderer.fileOpen() but that returns content.
// We might need a new IPC channel for picking a file path without reading it.

// However, looking at VSCode implementation, it calculates relative path.
// In browser/electron, we can do similar if we have node integration or IPC.
// Since we have node integration in main process, we can use window.ipcRenderer to ask for a file path.

export async function insertAssetReference(editor: monaco.editor.IStandaloneCodeEditor) {
    const adapter = new MonacoEditorAdapter(editor);
    
    // Request file selection from main process
    const filePath = await window.ipcRenderer.invoke<string | null>('dialog-show-open-dialog', {
        properties: ['openFile'],
        title: 'Select asset to insert'
    });

    if (!filePath) return;

    // We need the current document path to calculate relative path
    // The editor model uri in Monaco is usually file://...
    const docPath = editor.getModel()?.uri.fsPath;
    if (!docPath) return;

    // Calculate relative path via IPC to ensure correct path handling (windows vs posix)
    const relativePath = await window.ipcRenderer.invoke<string>('path-relative', docPath, filePath);

    await commands.InsertAssetCommand.execute(adapter, {
        path: relativePath,
        caption: ''
    });
}

export async function insertVerbatimBlock(editor: monaco.editor.IStandaloneCodeEditor) {
    const adapter = new MonacoEditorAdapter(editor);
    
    const filePath = await window.ipcRenderer.invoke<string | null>('dialog-show-open-dialog', {
        properties: ['openFile'],
        title: 'Select file to embed as verbatim block'
    });

    if (!filePath) return;

    const docPath = editor.getModel()?.uri.fsPath;
    if (!docPath) return;

    const relativePath = await window.ipcRenderer.invoke<string>('path-relative', docPath, filePath);
    const content = await window.ipcRenderer.invoke<string | null>('file-read', filePath);

    if (content === null) {
        console.error('Failed to read file content');
        return;
    }

    // Infer language from extension
    const ext = filePath.split('.').pop() || 'txt';
    const language = ext === 'py' ? 'python' : ext === 'js' ? 'javascript' : ext === 'ts' ? 'typescript' : ext;

    await commands.InsertVerbatimCommand.execute(adapter, {
        path: relativePath,
        content: content.trim(),
        language
    });
}
