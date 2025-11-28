import { ReactNode } from 'react';
import { FileTree } from './FileTree';

interface LayoutProps {
    children: ReactNode;
    sidebar?: ReactNode;
    panel?: ReactNode;
    rootPath?: string;
    onFileSelect: (path: string) => void;
}

export function Layout({ children, panel, rootPath, onFileSelect }: LayoutProps) {
    return (
        <div style={{ display: 'flex', width: '100vw', height: '100vh', overflow: 'hidden', backgroundColor: '#1e1e1e', color: '#cccccc' }}>
            {/* Sidebar */}
            <FileTree rootPath={rootPath} onFileSelect={onFileSelect} />

            {/* Main Content */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
                {children}
            </div>

            {/* Right Panel (Outline) */}
            {panel && (
                <div style={{
                    width: '200px',
                    borderLeft: '1px solid #1e1e1e',
                    backgroundColor: '#252526'
                }}>
                    {panel}
                </div>
            )}
        </div>
    );
}
