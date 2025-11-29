import { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';

interface FileEntry {
    name: string;
    isDirectory: boolean;
    path: string;
    children?: FileEntry[];
    isOpen?: boolean;
}

interface FileTreeProps {
    rootPath?: string;
    selectedFile?: string | null;
    onFileSelect: (path: string) => void;
}

export function FileTree({ rootPath, selectedFile, onFileSelect }: FileTreeProps) {
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
        return entries.map(entry => {
            const isSelected = !entry.isDirectory && entry.path === selectedFile;
            return (
                <div key={entry.path}>
                    <div
                        className={cn(
                            "cursor-pointer py-0.5 flex items-center text-[13px]",
                            "hover:bg-panel-hover",
                            isSelected
                                ? "bg-accent text-accent-foreground"
                                : "text-foreground"
                        )}
                        style={{ paddingLeft: `${depth * 10 + 10}px` }}
                        onClick={(e) => {
                            e.stopPropagation();
                            toggleDir(entry);
                        }}
                    >
                        <span className="mr-1.5 w-4 inline-block text-center">
                            {entry.isDirectory ? (entry.isOpen ? 'v' : '>') : ''}
                        </span>
                        {entry.name}
                    </div>
                    {entry.isOpen && entry.children && (
                        <div>{renderTree(entry.children, depth + 1)}</div>
                    )}
                </div>
            );
        });
    };

    return (
        <div className="h-full bg-panel overflow-y-auto text-foreground"
            style={{ fontFamily: 'system-ui, sans-serif' }}
        >
            <div className="p-2.5 text-xs font-bold uppercase tracking-wider border-b border-border">
                Explorer
            </div>
            {files.length > 0 ? renderTree(files) : (
                <div className="p-2.5 text-[13px] text-muted-foreground">
                    {rootPath ? 'Loading...' : 'No folder opened'}
                </div>
            )}
        </div>
    );
}
