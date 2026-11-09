import { useState } from "react";
import { Clock, Database, FilePlus2, RotateCcw, Trash2 } from "lucide-react";

import type { HistoryEntry, TableInfo } from "../../lib/types";

interface HistoryPanelProps {
  history: HistoryEntry[];
  tables: TableInfo[];
  onOpen: (query: string) => void;
  onRerun: (query: string) => void;
  onClear: () => void;
}

function preview(query: string) {
  const line = query.split("\n").find((part) => part.trim().length > 0) ?? query;
  return line.trim();
}

export function HistoryPanel({ history, tables, onOpen, onRerun, onClear }: HistoryPanelProps) {
  const [view, setView] = useState<"history" | "schema">("history");

  return (
    <div className="tool-window">
      <div className="tool-window-header">
        <span className="tool-window-title">{view === "history" ? (
          <>
            <Clock size={13} /> Query History
          </>
        ) : (
          <>
            <Database size={13} /> Schéma
          </>
        )}</span>
        {view === "history" && (
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
        )}
      </div>

      <div className="seg">
        <button
          type="button"
          className={view === "history" ? "on" : ""}
          onClick={() => setView("history")}
        >
          Historique
        </button>
        <button
          type="button"
          className={view === "schema" ? "on" : ""}
          onClick={() => setView("schema")}
        >
          Schéma
        </button>
      </div>

      {view === "history" && (
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
      )}

      {view === "schema" && (
        <div className="schema-list">
          {tables.length === 0 ? (
            <div className="panel-empty">
              <Database size={22} />
              <p>Aucune table dans ce workspace.</p>
            </div>
          ) : (
            tables.map((table) => (
              <div className="schema-row" key={table.name}>
                <Database size={13} />
                <span>{table.name}</span>
                <small>{table.columns.length} col.</small>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
