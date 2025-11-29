import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker&inline';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker&inline';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker&inline';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker&inline';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker&inline';

const DEBUG_LANG = 'lex';
const DEBUG_THEME = 'lex-theme';

// @ts-ignore
window.MonacoEnvironment = {
    getWorker(_: any, label: string) {
        if (label === 'json') {
            return new jsonWorker();
        }
        if (label === 'css' || label === 'scss' || label === 'less') {
            return new cssWorker();
        }
        if (label === 'html' || label === 'handlebars' || label === 'razor') {
            return new htmlWorker();
        }
        if (label === 'typescript' || label === 'javascript') {
            return new tsWorker();
        }
        return new editorWorker();
    }
};

export function initDebugMonaco() {
    // Register lex language
    monaco.languages.register({ id: DEBUG_LANG, extensions: ['.lex'] });

    monaco.languages.setLanguageConfiguration(DEBUG_LANG, {
        comments: {
            lineComment: '#',
        }
    });

    // Monarch provider for Session Title highlighting
    monaco.languages.setMonarchTokensProvider(DEBUG_LANG, {
        tokenizer: {
            root: [
                [/^Session Title.*/, 'sessionTitle'],
                [/^#.*/, 'comment'],
                [/[a-z]+/, 'string'],
            ]
        }
    });

    // Define Theme
    monaco.editor.defineTheme(DEBUG_THEME, {
        base: 'vs-dark',
        inherit: true,
        rules: [
            { token: 'sessionTitle', foreground: '#FF0000', fontStyle: 'bold' },
            { token: 'class', foreground: '#FF0000', fontStyle: 'bold' },
            { token: 'comment', foreground: '#888888' }
        ],
        colors: {},
        // @ts-ignore
        semanticHighlighting: true
    });
}
