import { FileCode2, Loader2, Play, Plus, X } from "lucide-react";

import type { QueryTab } from "../../lib/types";

interface EditorTabsProps {
  tabs: QueryTab[];
  activeTabId: string;
  running: boolean;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onNew: () => void;
  onRun: () => void;
}

export function EditorTabs({
  tabs,
  activeTabId,
  running,
  onSelect,
  onClose,
  onNew,
  onRun,
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
      <button
        type="button"
        className="tab-run"
        title="Exécuter la requête (Ctrl+Enter)"
        disabled={running}
        onClick={onRun}
      >
        {running ? <Loader2 size={14} className="spin" /> : <Play size={13} fill="currentColor" />}
        <span>{running ? "Exécution…" : "Run"}</span>
      </button>
    </div>
  );
}
