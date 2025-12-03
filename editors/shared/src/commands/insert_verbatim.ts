import { Command, EditorAdapter } from '../types.js';

export interface InsertVerbatimArgs {
    path: string;
}

export const InsertVerbatimCommand: Command<InsertVerbatimArgs> = {
    id: 'lex.insertVerbatim',
    execute: async (editor: EditorAdapter, args: InsertVerbatimArgs) => {
        const text = `![[${args.path}]]`;
        await editor.insertText(text);
    }
};
