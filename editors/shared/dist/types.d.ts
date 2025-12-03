export interface Range {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
}
export interface Position {
    line: number;
    column: number;
}
export interface EditorAdapter {
    /**
     * Inserts text at the current cursor position.
     */
    insertText(text: string): Promise<void>;
    /**
     * Replaces the text in the given range.
     */
    replaceRange(range: Range, text: string): Promise<void>;
    /**
     * Gets the full text content of the document.
     */
    getText(): Promise<string>;
    /**
     * Gets the text content within the specified range.
     */
    getTextInRange(range: Range): Promise<string>;
    /**
     * Gets the current selection range.
     */
    getSelection(): Promise<Range>;
    /**
     * Sets the selection range.
     */
    setSelection(range: Range): Promise<void>;
    /**
     * Gets the current cursor position.
     */
    getCursorPosition(): Promise<Position>;
}
export interface Command<TArgs = any> {
    id: string;
    execute(editor: EditorAdapter, args: TArgs): Promise<void>;
}
