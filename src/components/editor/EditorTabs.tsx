import { FileCode2, Plus, X } from "lucide-react";

import type { QueryTab } from "../../lib/types";

interface EditorTabsProps {
  tabs: QueryTab[];
  activeTabId: string;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onNew: () => void;
}

export function EditorTabs({
  tabs,
  activeTabId,
  onSelect,
  onClose,
  onNew,
}: EditorTabsProps) {
  return (
    <div className="editor-tabs">
      <div className="tab-strip">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`editor-tab ${tab.id === activeTabId ? "active" : ""}`}
            onClick={() => onSelect(tab.id)}
            role="tab"
            aria-selected={tab.id === activeTabId}
            tabIndex={0}
            onKeyDown={(event) => {
              if (event.key === "Enter") onSelect(tab.id);
            }}
          >
            <FileCode2 size={14} className="tab-icon" />
            <span className="tab-title">{tab.title}</span>
            <button
              type="button"
              className="tab-close"
              title="Fermer"
              onClick={(event) => {
                event.stopPropagation();
                onClose(tab.id);
              }}
            >
              <X size={12} />
            </button>
          </div>
        ))}
        <button type="button" className="new-tab-button" title="Nouvelle requête" onClick={onNew}>
          <Plus size={15} />
        </button>
      </div>
      <div className="editor-tab-meta">
        <span>FakeSQL</span>
        <span className="dot-separator" />
        <span>UTF-8</span>
      </div>
    </div>
  );
}
