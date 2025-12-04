export interface EditorSettings {
    showRuler: boolean;
    rulerWidth: number;
    vimMode: boolean;
    spellCheck: boolean;
    spellCheckLanguage: 'en_US' | 'pt_BR';
}

export interface FormatterSettings {
    sessionBlankLinesBefore: number;
    sessionBlankLinesAfter: number;
    normalizeSeqMarkers: boolean;
    unorderedSeqMarker: string;
    maxBlankLines: number;
    indentString: string;
    preserveTrailingBlanks: boolean;
    normalizeVerbatimMarkers: boolean;
    formatOnSave: boolean;
}

export interface AppSettings {
    editor: EditorSettings;
    formatter: FormatterSettings;
}

export const defaultEditorSettings: EditorSettings = {
    showRuler: false,
    rulerWidth: 100,
    vimMode: false,
    spellCheck: true,
    spellCheckLanguage: 'en_US',
};

export const defaultFormatterSettings: FormatterSettings = {
    sessionBlankLinesBefore: 1,
    sessionBlankLinesAfter: 1,
    normalizeSeqMarkers: true,
    unorderedSeqMarker: '-',
    maxBlankLines: 2,
    indentString: '    ',
    preserveTrailingBlanks: false,
    normalizeVerbatimMarkers: true,
    formatOnSave: false,
};

export const defaultAppSettings: AppSettings = {
    editor: defaultEditorSettings,
    formatter: defaultFormatterSettings,
};
