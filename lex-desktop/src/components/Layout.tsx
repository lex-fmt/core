import { ReactNode, useEffect, useState } from 'react';
import { FileTree } from './FileTree';

type ThemeMode = 'dark' | 'light';

const THEME_COLORS = {
    dark: {
        bg: '#1e1e1e',
        text: '#cccccc',
        panelBg: '#252526',
        border: '#1e1e1e',
    },
    light: {
        bg: '#ffffff',
        text: '#333333',
        panelBg: '#f3f3f3',
        border: '#e0e0e0',
    },
};

interface LayoutProps {
    children: ReactNode;
    sidebar?: ReactNode;
    panel?: ReactNode;
    rootPath?: string;
    onFileSelect: (path: string) => void;
}

export function Layout({ children, panel, rootPath, onFileSelect }: LayoutProps) {
    const [themeMode, setThemeMode] = useState<ThemeMode>('dark');

    useEffect(() => {
        // Get initial theme
        window.ipcRenderer.getNativeTheme().then((mode: ThemeMode) => {
            setThemeMode(mode);
        });

        // Listen for theme changes
        const unsubscribe = window.ipcRenderer.onNativeThemeChanged((mode: ThemeMode) => {
            setThemeMode(mode);
        });

        return unsubscribe;
    }, []);

    const colors = THEME_COLORS[themeMode];

    return (
        <div style={{ display: 'flex', width: '100vw', height: '100vh', overflow: 'hidden', backgroundColor: colors.bg, color: colors.text }}>
            {/* Sidebar */}
            <FileTree rootPath={rootPath} onFileSelect={onFileSelect} themeMode={themeMode} />

            {/* Main Content */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
                {children}
            </div>

            {/* Right Panel (Outline) */}
            {panel && (
                <div style={{
                    width: '200px',
                    borderLeft: `1px solid ${colors.border}`,
                    backgroundColor: colors.panelBg
                }}>
                    {panel}
                </div>
            )}
        </div>
    );
}
