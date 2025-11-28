import { useState, useEffect } from 'react';
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
                const uri = 'file://' + currentFile;
                // We might need to wait for the document to be opened in the LSP server.
                // A simple retry or delay might be needed if this runs too fast.
                // But since Editor opens it first, it should be fine if we are reactive to currentFile.

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

        // Poll for updates? Or listen to an event?
        // For now, let's just fetch once on file change. 
        // Ideally we'd re-fetch on save or debounce on change.
        const interval = setInterval(fetchSymbols, 2000);
        return () => clearInterval(interval);

    }, [currentFile]);

    const renderSymbols = (items: DocumentSymbol[], depth = 0) => {
        return items.map((item, index) => (
            <div key={index}>
                <div
                    style={{
                        paddingLeft: `${depth * 10 + 10}px`,
                        paddingTop: '2px',
                        paddingBottom: '2px',
                        color: '#cccccc',
                        fontSize: '12px',
                        whiteSpace: 'nowrap',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis'
                    }}
                    title={item.name}
                >
                    {/* We could add icons based on item.kind */}
                    {item.name}
                </div>
                {item.children && renderSymbols(item.children, depth + 1)}
            </div>
        ));
    };

    return (
        <div style={{ height: '100%', overflowY: 'auto', color: '#cccccc', fontFamily: 'system-ui, sans-serif' }}>
            <div style={{
                padding: '10px',
                fontSize: '11px',
                fontWeight: 'bold',
                textTransform: 'uppercase',
                letterSpacing: '1px',
                borderBottom: '1px solid #1e1e1e'
            }}>
                Outline
            </div>
            {symbols.length > 0 ? renderSymbols(symbols) : (
                <div style={{ padding: '10px', fontSize: '13px', color: '#888' }}>
                    {currentFile ? 'No symbols found' : 'No file open'}
                </div>
            )}
        </div>
    );
}
