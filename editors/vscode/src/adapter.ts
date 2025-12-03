import * as vscode from 'vscode';
import { EditorAdapter, Range, Position } from '@lex/shared';

export class VSCodeEditorAdapter implements EditorAdapter {
    constructor(private editor: vscode.TextEditor) {}

    async insertText(text: string): Promise<void> {
        await this.editor.edit(editBuilder => {
            editBuilder.insert(this.editor.selection.active, text);
        });
    }

    async replaceRange(range: Range, text: string): Promise<void> {
        const vscodeRange = new vscode.Range(
            range.startLine - 1,
            range.startColumn - 1,
            range.endLine - 1,
            range.endColumn - 1
        );
        await this.editor.edit(editBuilder => {
            editBuilder.replace(vscodeRange, text);
        });
    }

    async getText(): Promise<string> {
        return this.editor.document.getText();
    }

    async getTextInRange(range: Range): Promise<string> {
        const vscodeRange = new vscode.Range(
            range.startLine - 1,
            range.startColumn - 1,
            range.endLine - 1,
            range.endColumn - 1
        );
        return this.editor.document.getText(vscodeRange);
    }

    async getSelection(): Promise<Range> {
        const selection = this.editor.selection;
        return {
            startLine: selection.start.line + 1,
            startColumn: selection.start.character + 1,
            endLine: selection.end.line + 1,
            endColumn: selection.end.character + 1
        };
    }

    async setSelection(range: Range): Promise<void> {
        const start = new vscode.Position(range.startLine - 1, range.startColumn - 1);
        const end = new vscode.Position(range.endLine - 1, range.endColumn - 1);
        this.editor.selection = new vscode.Selection(start, end);
    }

    async getCursorPosition(): Promise<Position> {
        const position = this.editor.selection.active;
        return {
            line: position.line + 1,
            column: position.character + 1
        };
    }
}
