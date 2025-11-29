import { useState, useEffect } from 'react';
import { Uri } from 'monaco-editor';
import { lspClient } from '../lsp/client';

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
}

export function Outline({ currentFile }: OutlineProps) {
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

    const renderSymbols = (items: DocumentSymbol[], depth = 0) => {
        return items.map((item, index) => (
            <div key={index}>
                <div
                    className="text-foreground hover:bg-panel-hover cursor-pointer"
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
                >
                    {item.name}
                </div>
                {item.children && renderSymbols(item.children, depth + 1)}
            </div>
        ));
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
