import { X } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface Tab {
    id: string;
    path: string;
    name: string;
}

interface TabBarProps {
    tabs: Tab[];
    activeTabId: string | null;
    onTabSelect: (tabId: string) => void;
    onTabClose: (tabId: string) => void;
}

function truncateName(name: string, maxLength: number = 20): string {
    if (name.length <= maxLength) return name;
    return name.slice(0, maxLength - 1) + '\u2026'; // ellipsis character
}

export function TabBar({ tabs, activeTabId, onTabSelect, onTabClose }: TabBarProps) {
    if (tabs.length === 0) {
        return (
            <div className="h-9 bg-panel border-b border-border shrink-0" />
        );
    }

    return (
        <div className="h-9 flex items-end bg-panel border-b border-border shrink-0 overflow-x-auto overflow-y-hidden">
            {tabs.map(tab => (
                <div
                    key={tab.id}
                    data-testid="editor-tab"
                    data-tab-id={tab.id}
                    data-tab-path={tab.path}
                    data-active={activeTabId === tab.id}
                    className={cn(
                        "group flex items-center gap-1.5 h-8 px-3 cursor-pointer border-r border-border",
                        "hover:bg-panel-hover transition-colors",
                        activeTabId === tab.id
                            ? "bg-background text-foreground"
                            : "bg-faintest text-muted-foreground"
                    )}
                    onClick={() => onTabSelect(tab.id)}
                >
                    <span className="text-sm whitespace-nowrap">
                        {truncateName(tab.name)}
                    </span>
                    <button
                        className={cn(
                            "p-0.5 rounded hover:bg-border transition-colors",
                            "text-muted opacity-0 group-hover:opacity-100",
                            activeTabId === tab.id && "opacity-100"
                        )}
                        onClick={(e) => {
                            e.stopPropagation();
                            onTabClose(tab.id);
                        }}
                        title="Close"
                    >
                        <X size={14} />
                    </button>
                </div>
            ))}
        </div>
    );
}
