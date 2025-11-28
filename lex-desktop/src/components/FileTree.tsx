import { useState, useEffect } from 'react';

interface FileEntry {
    name: string;
    isDirectory: boolean;
    path: string;
    children?: FileEntry[];
    isOpen?: boolean;
}

interface FileTreeProps {
    rootPath?: string;
    onFileSelect: (path: string) => void;
}

export function FileTree({ rootPath, onFileSelect }: FileTreeProps) {
    const [files, setFiles] = useState<FileEntry[]>([]);

    useEffect(() => {
        if (rootPath) {
            loadDir(rootPath).then(setFiles);
        }
    }, [rootPath]);

    const loadDir = async (path: string): Promise<FileEntry[]> => {
        const entries = await window.ipcRenderer.fileReadDir(path);
        return entries.sort((a, b) => {
            if (a.isDirectory === b.isDirectory) {
                return a.name.localeCompare(b.name);
            }
            return a.isDirectory ? -1 : 1;
        });
    };

    const toggleDir = async (entry: FileEntry) => {
        if (!entry.isDirectory) {
            onFileSelect(entry.path);
            return;
        }

        if (entry.isOpen) {
            // Close
            const closeDir = (list: FileEntry[]): FileEntry[] => {
                return list.map(item => {
                    if (item.path === entry.path) {
                        return { ...item, isOpen: false };
                    }
                    if (item.children) {
                        return { ...item, children: closeDir(item.children) };
                    }
                    return item;
                });
            };
            setFiles(prev => closeDir(prev));
        } else {
            // Open
            const children = await loadDir(entry.path);
            const openDir = (list: FileEntry[]): FileEntry[] => {
                return list.map(item => {
                    if (item.path === entry.path) {
                        return { ...item, isOpen: true, children };
                    }
                    if (item.children) {
                        return { ...item, children: openDir(item.children) };
                    }
                    return item;
                });
            };
            setFiles(prev => openDir(prev));
        }
    };

    const renderTree = (entries: FileEntry[], depth = 0) => {
        return entries.map(entry => (
            <div key={entry.path}>
                <div
                    style={{
                        paddingLeft: `${depth * 10 + 10}px`,
                        cursor: 'pointer',
                        paddingTop: '2px',
                        paddingBottom: '2px',
                        color: '#cccccc',
                        backgroundColor: 'transparent',
                        display: 'flex',
                        alignItems: 'center',
                        fontSize: '13px'
                    }}
                    onClick={(e) => {
                        e.stopPropagation();
                        toggleDir(entry);
                    }}
                    onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#2a2d2e'}
                    onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'transparent'}
                >
                    <span style={{ marginRight: '5px', width: '16px', display: 'inline-block', textAlign: 'center' }}>
                        {entry.isDirectory ? (entry.isOpen ? 'v' : '>') : ''}
                    </span>
                    {entry.name}
                </div>
                {entry.isOpen && entry.children && (
                    <div>{renderTree(entry.children, depth + 1)}</div>
                )}
            </div>
        ));
    };

    return (
        <div style={{
            width: '250px',
            height: '100%',
            backgroundColor: '#252526',
            overflowY: 'auto',
            borderRight: '1px solid #1e1e1e',
            color: '#cccccc',
            fontFamily: 'system-ui, sans-serif'
        }}>
            <div style={{
                padding: '10px',
                fontSize: '11px',
                fontWeight: 'bold',
                textTransform: 'uppercase',
                letterSpacing: '1px'
            }}>
                Explorer
            </div>
            {files.length > 0 ? renderTree(files) : (
                <div style={{ padding: '10px', fontSize: '13px', color: '#888' }}>
                    {rootPath ? 'Loading...' : 'No folder opened'}
                </div>
            )}
        </div>
    );
}
