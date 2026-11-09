import { Database, FolderOpen, Loader2 } from "lucide-react";

import type { QueryResult } from "../lib/types";

interface StatusBarProps {
  workspace: string | null;
  tableCount: number;
  result: QueryResult | null;
  running: boolean;
}

export function StatusBar({ workspace, tableCount, result, running }: StatusBarProps) {
  return (
    <div className="status-bar">
      <div className="status-left">
        <span className={`status-dot ${workspace ? "online" : ""}`} />
        <FolderOpen size={12} />
        <span className="status-path" title={workspace ?? undefined}>
          {workspace ?? "Aucun dossier ouvert"}
        </span>
      </div>

      <div className="status-right">
        {running && (
          <span className="status-item">
            <Loader2 size={12} className="spin" /> Exécution…
          </span>
        )}
        {result && !running && (
          <span className={`status-item ${result.success ? "" : "error"}`}>
            {result.success
              ? `${result.rowCount ?? result.affected ?? 0} ligne(s) · ${result.elapsedMs} ms`
              : "Erreur"}
          </span>
        )}
        <span className="status-item">
          <Database size={12} /> {tableCount} table(s)
        </span>
        <span className="status-item">FakeSQL</span>
        <span className="status-item">UTF-8</span>
      </div>
    </div>
  );
}
