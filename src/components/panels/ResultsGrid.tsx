import { useEffect, useState } from "react";
import {
  CheckCircle2,
  Clock,
  Download,
  Grid3X3,
  Loader2,
  Rows3,
  XCircle,
} from "lucide-react";

import { api, isTauriRuntime } from "../../lib/tauri";
import type { QueryResult } from "../../lib/types";

interface ResultsGridProps {
  results: QueryResult[] | null;
  running: boolean;
}

type SortState = { column: number; direction: "asc" | "desc" } | null;

/** entier or reel values right-align and color like numbers. */
const NUMERIC_RE = /^-?\d+(?:\.\d+)?$/;

async function copyText(value: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch {
    return false;
  }
}

function escapeCsvValue(value: string): string {
  if (/["\r\n,;]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

function toCsv(columns: string[], rows: string[][]): string {
  const lines = [
    columns.map(escapeCsvValue).join(","),
    ...rows.map((row) => row.map((value) => escapeCsvValue(value ?? "")).join(",")),
  ];
  // BOM so Excel opens UTF-8 data correctly.
  return `\uFEFF${lines.join("\r\n")}`;
}

function csvFileName(result: QueryResult, index: number): string {
  const command = result.command.toLowerCase() || "resultat";
  return `${command}-${index + 1}.csv`;
}

function sortedRows(rows: string[][], sort: SortState): string[][] {
  if (!sort) return rows;
  const { column, direction } = sort;
  const factor = direction === "asc" ? 1 : -1;
  const numeric = rows.every((row) => NUMERIC_RE.test(row[column] ?? ""));
  return [...rows].sort((a, b) => {
    const left = a[column] ?? "";
    const right = b[column] ?? "";
    if (numeric) {
      return factor * (Number(left) - Number(right));
    }
    return factor * left.localeCompare(right);
  });
}

export function ResultsGrid({ results, running }: ResultsGridProps) {
  if (running) {
    return (
      <div className="panel-empty">
        <Loader2 size={22} className="spin" />
        <p>Exécution de la requête…</p>
      </div>
    );
  }

  if (!results) {
    return (
      <div className="panel-empty">
        <Grid3X3 size={24} />
        <p>Exécutez une requête pour afficher les résultats.</p>
        <span className="panel-hint">Ctrl+Enter</span>
      </div>
    );
  }

  return (
    <div className="results-root">
      {results.map((result, index) => (
        <ResultSection result={result} index={index} key={`${index}-${result.command}-${result.elapsedMs}-${result.message ?? ""}`} />
      ))}
    </div>
  );
}

interface ResultSectionProps {
  result: QueryResult;
  index: number;
}

function ResultSection({ result, index }: ResultSectionProps) {
  const [sort, setSort] = useState<SortState>(null);
  const [exportStatus, setExportStatus] = useState<string | null>(null);
  const [copiedCell, setCopiedCell] = useState<{ row: number; col: number } | null>(null);

  useEffect(() => {
    if (!copiedCell) return;
    const timer = setTimeout(() => setCopiedCell(null), 1200);
    return () => clearTimeout(timer);
  }, [copiedCell]);

  if (!result.success) {
    return (
      <div className="result-message-card error">
        <XCircle size={22} />
        <div>
          <h3>La requête a échoué</h3>
          <p>{result.message}</p>
          <span className="panel-hint">{result.elapsedMs} ms</span>
        </div>
      </div>
    );
  }

  if (!result.columns) {
    return (
      <div className="result-message-card success">
        <CheckCircle2 size={22} />
        <div>
          <h3>Requête exécutée</h3>
          <p>{result.message}</p>
          <span className="panel-hint">{result.elapsedMs} ms</span>
        </div>
      </div>
    );
  }

  const rows = sortedRows(result.rows ?? [], sort);

  const toggleSort = (column: number) => {
    setSort((previous) => {
      if (previous?.column !== column) return { column, direction: "asc" };
      if (previous.direction === "asc") return { column, direction: "desc" };
      return null;
    });
  };

  const exportCsv = async () => {
    setExportStatus(null);
    try {
      const path = await api.exportCsv(
        csvFileName(result, index),
        toCsv(result.columns ?? [], result.rows ?? []),
      );
      if (path) {
        setExportStatus(`Exporté : ${path}`);
      }
    } catch {
      setExportStatus("L'export a échoué");
    }
  };

  return (
    <div className="results-panel">
      <div className="results-summary">
        <span className="result-command">{result.command}</span>
        <span className="summary-item">
          <Rows3 size={12} /> {result.rowCount ?? rows.length} ligne(s)
        </span>
        <span className="summary-item">
          <Clock size={12} /> {result.elapsedMs} ms
        </span>
        {result.message && <span className="summary-message">{result.message}</span>}
        <button
          type="button"
          className="icon-button small export-button"
          title={isTauriRuntime() ? "Exporter en CSV" : "Nécessite l'application Tauri"}
          disabled={!isTauriRuntime()}
          onClick={() => void exportCsv()}
        >
          <Download size={13} />
        </button>
      </div>

      {exportStatus && <div className="export-status">{exportStatus}</div>}

      <div className="results-scroll">
        <table className="results-table">
          <thead>
            <tr>
              <th className="rownum-header">#</th>
              {result.columns.map((column, columnIndex) => (
                <th
                  key={column}
                  className="sortable"
                  title="Trier"
                  onClick={() => toggleSort(columnIndex)}
                >
                  {column}
                  {sort?.column === columnIndex && (
                    <span className="sort-indicator">
                      {sort.direction === "asc" ? "▲" : "▼"}
                    </span>
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 && (
              <tr>
                <td className="empty-cell" colSpan={result.columns.length + 1}>
                  Aucune ligne
                </td>
              </tr>
            )}
            {rows.map((row, rowIndex) => (
              <tr key={rowIndex}>
                <td className="rownum">{rowIndex + 1}</td>
                {result.columns?.map((column, columnIndex) => {
                  const value = row[columnIndex] ?? "";
                  const numeric = NUMERIC_RE.test(value);
                  const copied =
                    copiedCell?.row === rowIndex && copiedCell?.col === columnIndex;
                  return (
                    <td
                      key={`${rowIndex}-${column}`}
                      className={`${numeric ? "numeric" : ""} ${copied ? "copied" : ""}`}
                      title={
                        copied
                          ? "Copié dans le presse-papiers"
                          : "Cliquer pour copier la valeur"
                      }
                      onClick={() => {
                        void copyText(value).then((done) => {
                          if (done) setCopiedCell({ row: rowIndex, col: columnIndex });
                        });
                      }}
                    >
                      {value}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
