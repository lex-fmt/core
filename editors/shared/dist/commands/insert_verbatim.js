export const InsertVerbatimCommand = {
    id: 'lex.insertVerbatim',
    execute: async (editor, args) => {
        const text = `![[${args.path}]]`;
        await editor.insertText(text);
    }
};
