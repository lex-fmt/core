import { createContext, useContext, useEffect, useState, ReactNode } from 'react';

interface EditorSettings {
    showRuler: boolean;
    rulerWidth: number;
    vimMode: boolean;
}

interface AppSettings {
    editor: EditorSettings;
}

interface SettingsContextType {
    settings: AppSettings;
    updateEditorSettings: (settings: EditorSettings) => Promise<void>;
}

const defaultSettings: AppSettings = {
    editor: {
        showRuler: false,
        rulerWidth: 100,
        vimMode: false,
    },
};

const SettingsContext = createContext<SettingsContextType | null>(null);

export function useSettings() {
    const context = useContext(SettingsContext);
    if (!context) {
        throw new Error('useSettings must be used within a SettingsProvider');
    }
    return context;
}

export function SettingsProvider({ children }: { children: ReactNode }) {
    const [settings, setSettings] = useState<AppSettings>(defaultSettings);

    useEffect(() => {
        // Initial load
        window.ipcRenderer.getAppSettings().then((loadedSettings: any) => {
            setSettings(prev => ({
                ...prev,
                ...loadedSettings,
                editor: { ...prev.editor, ...loadedSettings.editor }
            }));
        });

        // Listen for changes
        const unsubscribe = window.ipcRenderer.onSettingsChanged((newSettings: any) => {
            setSettings(prev => ({
                ...prev,
                ...newSettings,
                editor: { ...prev.editor, ...newSettings.editor }
            }));
        });

        return unsubscribe;
    }, []);

    const updateEditorSettings = async (editorSettings: EditorSettings) => {
        await window.ipcRenderer.setEditorSettings(editorSettings);
        // Optimistic update
        setSettings(prev => ({
            ...prev,
            editor: editorSettings
        }));
    };

    return (
        <SettingsContext.Provider value={{ settings, updateEditorSettings }}>
            {children}
        </SettingsContext.Provider>
    );
}
