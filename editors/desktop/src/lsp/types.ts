export interface LspPosition {
  line: number;
  character: number;
}

export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

export interface LspTextEdit {
  range: LspRange;
  newText: string;
}

export interface LspCompletionItem {
  label: string;
  kind?: number;
  insertText?: string;
  detail?: string;
  documentation?: string | { value: string };
  textEdit?: LspTextEdit & { newText?: string };
}

export type LspCompletionResponse =
  | LspCompletionItem[]
  | { items: LspCompletionItem[] };

export interface LspFormattingEdit {
  range: LspRange;
  newText: string;
}

export interface LexInsertResponse {
  text: string;
  cursorOffset: number;
}
