import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';

interface SnippetInsertionPayload {
    text: string;
    cursorOffset: number;
}

function isSnippetInsertionPayload(value: unknown): value is SnippetInsertionPayload {
    return (
        typeof value === 'object' &&
        value !== null &&
        typeof (value as { text?: unknown }).text === 'string' &&
        typeof (value as { cursorOffset?: unknown }).cursorOffset === 'number'
    );
}

async function invokeInsertCommand(
    editor: monaco.editor.IStandaloneCodeEditor,
    command: string,
    args: any[]
): Promise<void> {
    const model = editor.getModel();
    if (!model) return;

    const position = editor.getPosition();
    if (!position) return;

    try {
        const response = await lspClient.sendRequest('workspace/executeCommand', {
            command,
            arguments: [model.uri.toString(), { line: position.lineNumber - 1, character: position.column - 1 }, ...args]
        });

        if (response && isSnippetInsertionPayload(response)) {
            console.log('invokeInsertCommand: received snippet payload', response);
            insertSnippet(editor, position, response);
        } else {
            console.error('Invalid snippet payload received', response);
        }
    } catch (error) {
        console.error(`Failed to execute ${command}:`, error);
        throw error;
    }
}

function insertSnippet(
    editor: monaco.editor.IStandaloneCodeEditor,
    position: monaco.Position,
    payload: SnippetInsertionPayload
) {
    const prefix = position.lineNumber === 1 && position.column === 1 ? '' : '\n';
    const suffix = '\n';
    const textToInsert = `${prefix}${payload.text}${suffix}`;

    editor.executeEdits('lex-insert', [{
        range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
        text: textToInsert,
        forceMoveMarkers: true
    }]);

    // Calculate new cursor position
    // Note: This is a simplified calculation. For robust offset handling, we might need model.getPositionAt(offset).
    // But since we just inserted text, we can calculate the offset.
    
    const model = editor.getModel();
    if (model) {
        const startOffset = model.getOffsetAt(position);
        const newCursorOffset = startOffset + prefix.length + payload.cursorOffset;
        const newPosition = model.getPositionAt(newCursorOffset);
        
        editor.setSelection(new monaco.Selection(
            newPosition.lineNumber, newPosition.column,
            newPosition.lineNumber, newPosition.column
        ));
        editor.revealPosition(newPosition);
    }
}

export async function insertAsset(editor: monaco.editor.IStandaloneCodeEditor, assetPath: string) {
    console.log('insertAsset called with', assetPath);
    await invokeInsertCommand(editor, 'lex.insert_asset', [assetPath]);
}

export async function insertVerbatim(editor: monaco.editor.IStandaloneCodeEditor, filePath: string) {
    await invokeInsertCommand(editor, 'lex.insert_verbatim', [filePath]);
}

export async function resolveAnnotation(editor: monaco.editor.IStandaloneCodeEditor) {
    await invokeInsertCommand(editor, 'lex.resolve_annotation', []);
}

export async function toggleAnnotations(editor: monaco.editor.IStandaloneCodeEditor) {
    await invokeInsertCommand(editor, 'lex.toggle_annotations', []);
}
