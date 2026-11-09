import { Clock, FilePlus2, RotateCcw, Trash2 } from "lucide-react";

import type { HistoryEntry } from "../../lib/types";

interface HistoryPanelProps {
  history: HistoryEntry[];
  onOpen: (query: string) => void;
  onRerun: (query: string) => void;
  onClear: () => void;
}

function preview(query: string) {
  const line = query.split("\n").find((part) => part.trim().length > 0) ?? query;
  return line.trim();
}

export function HistoryPanel({ history, onOpen, onRerun, onClear }: HistoryPanelProps) {
  return (
    <div className="tool-window">
      <div className="tool-window-header">
        <span className="tool-window-title">
          <Clock size={13} /> Query History
        </span>
        <div className="tool-window-actions">
          <button
            type="button"
            className="icon-button small"
            title="Effacer l'historique"
            onClick={onClear}
            disabled={history.length === 0}
          >
            <Trash2 size={13} />
          </button>
        </div>
      </div>

      <div className="history-list">
        {history.length === 0 && (
          <div className="panel-empty">
            <Clock size={22} />
            <p>Aucune requête exécutée pour le moment.</p>
            <span className="panel-hint">Ctrl+Enter pour lancer une requête</span>
          </div>
        )}

        {history.map((entry) => (
          <div className={`history-item ${entry.success ? "success" : "error"}`} key={entry.id}>
            <div className="history-top">
              <span className={`history-dot ${entry.success ? "success" : "error"}`} />
              <span className="history-time">{entry.time}</span>
              {entry.elapsedMs != null && (
                <span className="history-ms">{entry.elapsedMs} ms</span>
              )}
              <div className="history-actions">
                <button
                  type="button"
                  className="icon-button small"
                  title="Ouvrir dans une nouvelle requête"
                  onClick={() => onOpen(entry.query)}
                >
                  <FilePlus2 size={12} />
                </button>
                <button
                  type="button"
                  className="icon-button small"
                  title="Exécuter à nouveau"
                  onClick={() => onRerun(entry.query)}
                >
                  <RotateCcw size={12} />
                </button>
              </div>
            </div>
            <div className="history-query" title={entry.query}>
              {preview(entry.query)}
            </div>
            {entry.message && <div className="history-message">{entry.message}</div>}
          </div>
        ))}
      </div>
    </div>
  );
}
