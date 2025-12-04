import { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useSettings } from '@/contexts/SettingsContext';

interface SettingsDialogProps {
    isOpen: boolean;
    onClose: () => void;
}

export function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
    const { settings, updateEditorSettings } = useSettings();
    const [localSettings, setLocalSettings] = useState(settings.editor);

    useEffect(() => {
        setLocalSettings(settings.editor);
    }, [settings.editor, isOpen]);

    if (!isOpen) return null;

    const handleSave = () => {
        updateEditorSettings(localSettings);
        onClose();
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div className="w-[500px] bg-panel border border-border rounded-lg shadow-xl flex flex-col">
                <div className="flex items-center justify-between px-4 py-3 border-b border-border">
                    <h2 className="text-sm font-semibold">Settings</h2>
                    <button onClick={onClose} className="p-1 hover:bg-panel-hover rounded">
                        <X size={16} />
                    </button>
                </div>

                <div className="p-4 space-y-6">
                    <div className="space-y-4">
                        <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Editor</h3>

                        <div className="flex items-center justify-between">
                            <label htmlFor="show-ruler" className="text-sm">Show vertical col width ruler</label>
                            <input
                                id="show-ruler"
                                type="checkbox"
                                checked={localSettings.showRuler}
                                onChange={(e) => setLocalSettings(prev => ({ ...prev, showRuler: e.target.checked }))}
                                className="accent-primary"
                            />
                        </div>

                        <div className="flex items-center justify-between">
                            <label htmlFor="ruler-width" className={cn("text-sm", !localSettings.showRuler && "opacity-50")}>
                                Ruler width
                            </label>
                            <input
                                id="ruler-width"
                                type="number"
                                value={localSettings.rulerWidth}
                                disabled={!localSettings.showRuler}
                                onChange={(e) => setLocalSettings(prev => ({ ...prev, rulerWidth: parseInt(e.target.value) || 0 }))}
                                className="w-20 px-2 py-1 text-sm bg-input border border-border rounded focus:outline-none focus:border-primary disabled:opacity-50"
                            />
                        </div>
                    </div>

                    <div className="flex items-center justify-between">
                        <label htmlFor="vim-mode" className="text-sm">Enable Vim Mode</label>
                        <input
                            type="checkbox"
                            id="vim-mode"
                            checked={localSettings.vimMode}
                            onChange={(e) => setLocalSettings(prev => ({ ...prev, vimMode: e.target.checked }))}
                            className="accent-primary"
                        />
                    </div>
                </div>

                <div className="flex items-center justify-end gap-2 px-4 py-3 border-t border-border bg-panel-hover/30">
                    <button
                        onClick={onClose}
                        className="px-3 py-1.5 text-sm hover:bg-panel-hover rounded"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={handleSave}
                        className="px-3 py-1.5 text-sm bg-primary text-primary-foreground hover:bg-primary/90 rounded"
                    >
                        Save Changes
                    </button>
                </div>
            </div>
        </div>
    );
}
