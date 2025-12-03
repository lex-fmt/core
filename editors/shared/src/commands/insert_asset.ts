import { Command, EditorAdapter } from '../types.js';

export interface InsertAssetArgs {
    path: string;
    caption?: string;
}

export const InsertAssetCommand: Command<InsertAssetArgs> = {
    id: 'lex.insertAsset',
    execute: async (editor: EditorAdapter, args: InsertAssetArgs) => {
        const text = `![${args.caption || ''}](${args.path})`;
        await editor.insertText(text);
    }
};
