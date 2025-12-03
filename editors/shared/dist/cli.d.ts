export interface ConvertOptions {
    cliBinaryPath: string;
    fromFormat: 'lex' | 'markdown';
    toFormat: 'lex' | 'markdown' | 'html';
    targetLanguageId: string;
}
export declare function convertDocument(content: string, options: ConvertOptions): Promise<string>;
/**
 * Convert a Lex document to PDF and write it to the specified output file.
 * PDF requires file-based output since it's binary.
 */
export declare function convertToPdfFile(content: string, cliBinaryPath: string, outputPath: string): Promise<void>;
/**
 * Convert Lex content to HTML. Used by both export command and live preview.
 */
export declare function convertToHtml(content: string, cliBinaryPath: string): Promise<string>;
export interface ConvertFileOptions {
    cliBinaryPath: string;
    sourcePath: string;
    outputPath: string;
    toFormat: 'lex' | 'markdown' | 'html' | 'pdf';
}
export declare function convertFile(options: ConvertFileOptions): Promise<void>;
