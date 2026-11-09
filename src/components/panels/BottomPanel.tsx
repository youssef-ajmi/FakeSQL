import { Eraser, Grid3X3, Loader2, Terminal } from "lucide-react";

import type { BottomTab, ConsoleEntry, QueryResult } from "../../lib/types";
import { ConsolePanel } from "./ConsolePanel";
import { ResultsGrid } from "./ResultsGrid";

interface BottomPanelProps {
  tab: BottomTab;
  onChange: (tab: BottomTab) => void;
  results: QueryResult[] | null;
  entries: ConsoleEntry[];
  running: boolean;
  onClearConsole: () => void;
}

export function BottomPanel({
  tab,
  onChange,
  results,
  entries,
  running,
  onClearConsole,
}: BottomPanelProps) {
  const last = results && results.length > 0 ? results[results.length - 1] : null;
  const badge = last?.rowCount ?? last?.affected ?? null;

  return (
    <div className="bottom-panel-inner">
      <div className="panel-tabs">
        <button
          type="button"
          className={`panel-tab ${tab === "results" ? "active" : ""}`}
          onClick={() => onChange("results")}
        >
          <Grid3X3 size={13} /> Results
          {results && badge != null && <span className="tab-badge">{badge}</span>}
        </button>
        <button
          type="button"
          className={`panel-tab ${tab === "console" ? "active" : ""}`}
          onClick={() => onChange("console")}
        >
          <Terminal size={13} /> Console
          {entries.length > 0 && <span className="tab-badge">{entries.length}</span>}
        </button>

        <div className="panel-tabs-spacer" />

        {running && (
          <span className="panel-running">
            <Loader2 size={12} className="spin" /> Running
          </span>
        )}

        {tab === "console" && (
          <button
            type="button"
            className="icon-button small"
            title="Effacer la console"
            onClick={onClearConsole}
            disabled={entries.length === 0}
          >
            <Eraser size={13} />
          </button>
        )}
      </div>

      <div className="panel-content">
        {tab === "results" ? (
          <ResultsGrid results={results} running={running} />
        ) : (
          <ConsolePanel entries={entries} />
        )}
      </div>
    </div>
  );
}
