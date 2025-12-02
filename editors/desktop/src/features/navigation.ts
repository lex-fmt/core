import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';

export interface Location {
    uri: string;
    range: {
        start: { line: number; character: number };
        end: { line: number; character: number };
    };
}

async function invokeNavigationCommand(
    editor: monaco.editor.IStandaloneCodeEditor,
    command: string
): Promise<Location | null> {
    const model = editor.getModel();
    if (!model) return null;

    const position = editor.getPosition();
    if (!position) return null;

    try {
        const response = await lspClient.sendRequest('workspace/executeCommand', {
            command,
            arguments: [model.uri.toString(), { line: position.lineNumber - 1, character: position.column - 1 }]
        });

        if (response && typeof response === 'object' && 'uri' in response && 'range' in response) {
            return response as Location;
        }
    } catch (error) {
        console.error(`Failed to execute ${command}:`, error);
    }
    return null;
}

export async function nextAnnotation(editor: monaco.editor.IStandaloneCodeEditor): Promise<Location | null> {
    return invokeNavigationCommand(editor, 'lex.next_annotation');
}

export async function previousAnnotation(editor: monaco.editor.IStandaloneCodeEditor): Promise<Location | null> {
    return invokeNavigationCommand(editor, 'lex.previous_annotation');
}
