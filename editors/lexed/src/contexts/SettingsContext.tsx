import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { AppSettings, EditorSettings, FormatterSettings, defaultAppSettings } from '@/settings/types';
import { setSettingsSnapshot } from '@/settings/snapshot';

interface SettingsContextType {
    settings: AppSettings;
    updateEditorSettings: (settings: EditorSettings) => Promise<void>;
    updateFormatterSettings: (settings: FormatterSettings) => Promise<void>;
}

const defaultSettings: AppSettings = defaultAppSettings;

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
            setSettings(prev => {
                const next = {
                    editor: { ...prev.editor, ...(loadedSettings?.editor ?? {}) },
                    formatter: { ...prev.formatter, ...(loadedSettings?.formatter ?? {}) },
                } satisfies AppSettings;
                setSettingsSnapshot(next);
                return next;
            });
        });

        // Listen for changes
        const unsubscribe = window.ipcRenderer.onSettingsChanged((newSettings: any) => {
            setSettings(prev => {
                const next = {
                    editor: { ...prev.editor, ...(newSettings?.editor ?? {}) },
                    formatter: { ...prev.formatter, ...(newSettings?.formatter ?? {}) },
                } satisfies AppSettings;
                setSettingsSnapshot(next);
                return next;
            });
        });

        return unsubscribe;
    }, []);

    const updateEditorSettings = async (editorSettings: EditorSettings) => {
        await window.ipcRenderer.setEditorSettings(editorSettings);
        // Optimistic update
        setSettings(prev => {
            const next = { ...prev, editor: editorSettings } satisfies AppSettings;
            setSettingsSnapshot(next);
            return next;
        });
    };

    const updateFormatterSettings = async (formatterSettings: FormatterSettings) => {
        await window.ipcRenderer.setFormatterSettings(formatterSettings);
        setSettings(prev => {
            const next = { ...prev, formatter: formatterSettings } satisfies AppSettings;
            setSettingsSnapshot(next);
            return next;
        });
    };

    return (
        <SettingsContext.Provider value={{ settings, updateEditorSettings, updateFormatterSettings }}>
            {children}
        </SettingsContext.Provider>
    );
}
