import * as monaco from 'monaco-editor';
import { EditorAdapter, Range, Position } from '@lex/shared';

export class MonacoEditorAdapter implements EditorAdapter {
    constructor(private editor: monaco.editor.IStandaloneCodeEditor) {}

    async insertText(text: string): Promise<void> {
        const selection = this.editor.getSelection();
        if (!selection) return;

        const op = {
            range: selection,
            text: text,
            forceMoveMarkers: true
        };
        this.editor.executeEdits('lex-adapter', [op]);
    }

    async replaceRange(range: Range, text: string): Promise<void> {
        const monacoRange = new monaco.Range(
            range.startLine,
            range.startColumn,
            range.endLine,
            range.endColumn
        );
        const op = {
            range: monacoRange,
            text: text,
            forceMoveMarkers: true
        };
        this.editor.executeEdits('lex-adapter', [op]);
    }

    async getText(): Promise<string> {
        return this.editor.getValue();
    }

    async getTextInRange(range: Range): Promise<string> {
        const monacoRange = new monaco.Range(
            range.startLine,
            range.startColumn,
            range.endLine,
            range.endColumn
        );
        return this.editor.getModel()?.getValueInRange(monacoRange) || '';
    }

    async getSelection(): Promise<Range> {
        const selection = this.editor.getSelection();
        if (!selection) {
            return { startLine: 1, startColumn: 1, endLine: 1, endColumn: 1 };
        }
        return {
            startLine: selection.startLineNumber,
            startColumn: selection.startColumn,
            endLine: selection.endLineNumber,
            endColumn: selection.endColumn
        };
    }

    async setSelection(range: Range): Promise<void> {
        const monacoRange = new monaco.Range(
            range.startLine,
            range.startColumn,
            range.endLine,
            range.endColumn
        );
        this.editor.setSelection(monacoRange);
    }

    async getCursorPosition(): Promise<Position> {
        const position = this.editor.getPosition();
        if (!position) {
            return { line: 1, column: 1 };
        }
        return {
            line: position.lineNumber,
            column: position.column
        };
    }
}
