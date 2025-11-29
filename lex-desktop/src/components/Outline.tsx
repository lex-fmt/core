import { useState, useEffect, useMemo } from 'react';
import { Uri } from 'monaco-editor';
import type * as Monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';
import { cn } from '@/lib/utils';

interface DocumentSymbol {
    name: string;
    kind: number;
    range: Range;
    selectionRange: Range;
    children?: DocumentSymbol[];
}

interface Range {
    start: { line: number; character: number };
    end: { line: number; character: number };
}

interface OutlineProps {
    currentFile: string | null;
    editor?: Monaco.editor.IStandaloneCodeEditor | null;
    cursorLine?: number;
}

// Find the deepest symbol that contains the cursor line
function findActiveSymbol(symbols: DocumentSymbol[], line: number): DocumentSymbol | null {
    for (const symbol of symbols) {
        if (line >= symbol.range.start.line && line <= symbol.range.end.line) {
            // Check children for a more specific match
            if (symbol.children) {
                const childMatch = findActiveSymbol(symbol.children, line);
                if (childMatch) return childMatch;
            }
            return symbol;
        }
    }
    return null;
}

export function Outline({ currentFile, editor, cursorLine }: OutlineProps) {
    const [symbols, setSymbols] = useState<DocumentSymbol[]>([]);

    useEffect(() => {
        if (!currentFile) {
            setSymbols([]);
            return;
        }

        const fetchSymbols = async () => {
            try {
                const uri = Uri.file(currentFile).toString();
                const response = await lspClient.sendRequest('textDocument/documentSymbol', {
                    textDocument: { uri }
                });

                if (Array.isArray(response)) {
                    setSymbols(response);
                } else {
                    setSymbols([]);
                }
            } catch (e) {
                console.error('Failed to fetch symbols', e);
                setSymbols([]);
            }
        };

        fetchSymbols();
        const interval = setInterval(fetchSymbols, 2000);
        return () => clearInterval(interval);

    }, [currentFile]);

    // Find active symbol based on cursor position
    const activeSymbol = useMemo(() => {
        if (cursorLine === undefined) return null;
        return findActiveSymbol(symbols, cursorLine);
    }, [symbols, cursorLine]);

    const handleSymbolClick = (symbol: DocumentSymbol) => {
        if (!editor) return;

        // Navigate to the symbol's selection range (more precise than range)
        const position = {
            lineNumber: symbol.selectionRange.start.line + 1, // Monaco uses 1-based lines
            column: symbol.selectionRange.start.character + 1
        };

        editor.setPosition(position);
        editor.revealPositionInCenter(position);
        editor.focus();
    };

    const renderSymbols = (items: DocumentSymbol[], depth = 0) => {
        return items.map((item, index) => {
            const isActive = activeSymbol === item;
            return (
                <div key={index}>
                    <div
                        className={cn(
                            "cursor-pointer",
                            "hover:bg-panel-hover",
                            isActive
                                ? "bg-accent text-accent-foreground"
                                : "text-foreground"
                        )}
                        style={{
                            paddingLeft: `${depth * 10 + 10}px`,
                            paddingTop: '2px',
                            paddingBottom: '2px',
                            fontSize: '12px',
                            whiteSpace: 'nowrap',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis'
                        }}
                        title={item.name}
                        onClick={() => handleSymbolClick(item)}
                    >
                        {item.name}
                    </div>
                    {item.children && renderSymbols(item.children, depth + 1)}
                </div>
            );
        });
    };

    return (
        <div
            data-testid="outline-view"
            className="h-full overflow-y-auto text-foreground bg-panel"
            style={{ fontFamily: 'system-ui, sans-serif' }}
        >
            {symbols.length > 0 ? renderSymbols(symbols) : (
                <div className="p-2.5 text-sm text-muted-foreground">
                    {currentFile ? 'No symbols found' : 'No file open'}
                </div>
            )}
        </div>
    );
}
