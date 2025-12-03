import * as monaco from 'monaco-editor';

export type ThemeMode = 'dark' | 'light';

const LIGHT_COLORS = {
  normal: '#000000',
  muted: '#808080',
  faint: '#b3b3b3',
  faintest: '#cacaca',
  background: '#ffffff',
};

const DARK_COLORS = {
  normal: '#e0e0e0',
  muted: '#888888',
  faint: '#666666',
  faintest: '#555555',
  background: '#1e1e1e',
};

const THEME_NAME = 'lex-monochrome';
let themesDefined = false;

function defineThemes() {
  if (themesDefined) return;

  const define = (mode: ThemeMode) => {
    const colors = mode === 'dark' ? DARK_COLORS : LIGHT_COLORS;
    const baseTheme = mode === 'dark' ? 'vs-dark' : 'vs';

    monaco.editor.defineTheme(`${THEME_NAME}-${mode}`, {
      base: baseTheme,
      inherit: true,
      rules: [
        { token: 'SessionTitleText', foreground: colors.normal.replace('#', ''), fontStyle: 'bold' },
        { token: 'DefinitionSubject', foreground: colors.normal.replace('#', ''), fontStyle: 'italic' },
        { token: 'DefinitionContent', foreground: colors.normal.replace('#', '') },
        { token: 'InlineStrong', foreground: colors.normal.replace('#', ''), fontStyle: 'bold' },
        { token: 'InlineEmphasis', foreground: colors.normal.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineCode', foreground: colors.normal.replace('#', '') },
        { token: 'InlineMath', foreground: colors.normal.replace('#', ''), fontStyle: 'italic' },
        { token: 'VerbatimContent', foreground: colors.normal.replace('#', '') },
        { token: 'ListItemText', foreground: colors.normal.replace('#', '') },
        { token: 'DocumentTitle', foreground: colors.muted.replace('#', ''), fontStyle: 'bold' },
        { token: 'SessionMarker', foreground: colors.muted.replace('#', ''), fontStyle: 'italic' },
        { token: 'ListMarker', foreground: colors.muted.replace('#', ''), fontStyle: 'italic' },
        { token: 'Reference', foreground: colors.muted.replace('#', ''), fontStyle: 'underline' },
        { token: 'ReferenceCitation', foreground: colors.muted.replace('#', ''), fontStyle: 'underline' },
        { token: 'ReferenceFootnote', foreground: colors.muted.replace('#', ''), fontStyle: 'underline' },
        { token: 'AnnotationLabel', foreground: colors.faint.replace('#', '') },
        { token: 'AnnotationParameter', foreground: colors.faint.replace('#', '') },
        { token: 'AnnotationContent', foreground: colors.faint.replace('#', '') },
        { token: 'VerbatimSubject', foreground: colors.faint.replace('#', '') },
        { token: 'VerbatimLanguage', foreground: colors.faint.replace('#', '') },
        { token: 'VerbatimAttribute', foreground: colors.faint.replace('#', '') },
        { token: 'InlineMarker_strong_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_strong_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_emphasis_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_emphasis_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_code_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_code_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_math_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_math_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_ref_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        { token: 'InlineMarker_ref_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
      ],
      colors: {
        'editor.foreground': colors.normal,
        'editor.background': colors.background,
        'editorLineNumber.foreground': colors.faint,
        'editorLineNumber.activeForeground': colors.normal,
      },
    });
  };

  define('dark');
  define('light');
  themesDefined = true;
}

export function applyLexTheme(mode: ThemeMode) {
  defineThemes();
  const target = `${THEME_NAME}-${mode}`;
  monaco.editor.setTheme(target);
}

export function getThemeNameForMode(mode: ThemeMode) {
  defineThemes();
  return `${THEME_NAME}-${mode}`;
}
