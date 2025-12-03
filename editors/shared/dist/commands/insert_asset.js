export const InsertAssetCommand = {
    id: 'lex.insertAsset',
    execute: async (editor, args) => {
        const text = `![${args.caption || ''}](${args.path})`;
        await editor.insertText(text);
    }
};
