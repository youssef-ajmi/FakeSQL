import { useState } from "react";
import {
  ChevronDown,
  ChevronRight,
  Database,
  FolderOpen,
  FolderSearch,
  Hash,
  KeyRound,
  Plus,
  RefreshCw,
  Search,
  Table2,
  Trash2,
  Type,
} from "lucide-react";

import type { TableInfo } from "../../lib/types";

interface TablesPanelProps {
  tables: TableInfo[];
  hasWorkspace: boolean;
  onRefresh: () => void;
  onOpenFolder: () => void;
  onOpenTable: (name: string) => void;
  onDeleteTable: (name: string) => void;
  onCreateTable: () => void;
}

export function TablesPanel({
  tables,
  hasWorkspace,
  onRefresh,
  onOpenFolder,
  onOpenTable,
  onDeleteTable,
  onCreateTable,
}: TablesPanelProps) {
  const [filter, setFilter] = useState("");
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const visible = tables.filter((table) =>
    table.name.toLowerCase().includes(filter.trim().toLowerCase()),
  );

  const toggle = (name: string) => {
    setExpanded((previous) => {
      const next = new Set(previous);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  return (
    <div className="tool-window">
      <div className="tool-window-header">
        <span className="tool-window-title">
          <Database size={13} /> Database
        </span>
        <div className="tool-window-actions">
          <button
            type="button"
            className="icon-button small"
            title="Créer une table"
            onClick={onCreateTable}
            disabled={!hasWorkspace}
          >
            <Plus size={14} />
          </button>
          <button
            type="button"
            className="icon-button small"
            title="Rafraîchir"
            onClick={onRefresh}
            disabled={!hasWorkspace}
          >
            <RefreshCw size={13} />
          </button>
        </div>
      </div>

      <div className="tree-search">
        <Search size={13} />
        <input
          value={filter}
          onChange={(event) => setFilter(event.target.value)}
          placeholder="Filtrer les tables"
          spellCheck={false}
        />
      </div>

      <div className="tree" role="tree">
        {visible.length === 0 && (
          <div className="tree-empty">
            {hasWorkspace ? (
              <>
                <FolderSearch size={20} className="tree-empty-icon" />
                <p>Aucune table dans ce workspace.</p>
                <button type="button" className="secondary-button" onClick={onCreateTable}>
                  <Plus size={13} /> Créer une table
                </button>
              </>
            ) : (
              <>
                <FolderSearch size={20} className="tree-empty-icon" />
                <p>Ouvrez un dossier workspace pour voir vos tables.</p>
                <button type="button" className="secondary-button" onClick={onOpenFolder}>
                  <FolderOpen size={13} /> Ouvrir un dossier
                </button>
              </>
            )}
          </div>
        )}

        {visible.map((table) => {
          const isOpen = expanded.has(table.name);
          return (
            <div key={table.name} className="tree-group">
              <div
                className="tree-row"
                role="treeitem"
                aria-expanded={isOpen}
                tabIndex={0}
                onDoubleClick={() => onOpenTable(table.name)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") onOpenTable(table.name);
                  if (event.key === " ") {
                    event.preventDefault();
                    toggle(table.name);
                  }
                }}
              >
                <button
                  type="button"
                  className="tree-chevron"
                  onClick={() => toggle(table.name)}
                  title={isOpen ? "Réduire" : "Développer"}
                >
                  {isOpen ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
                </button>
                <Table2 size={14} className="tree-icon table" />
                <span className="tree-label">{table.name}</span>
                <span className="tree-badge">{table.rowCount}</span>
                <button
                  type="button"
                  className="tree-action"
                  title={`Supprimer ${table.name}`}
                  onClick={(event) => {
                    event.stopPropagation();
                    onDeleteTable(table.name);
                  }}
                >
                  <Trash2 size={12} />
                </button>
              </div>

              {isOpen && (
                <div className="tree-children">
                  {table.columns.map((column) => {
                    const isId = column.name === "id";
                    return (
                      <div className="tree-row column" key={column.name}>
                        {isId ? (
                          <KeyRound size={12} className="tree-icon key" />
                        ) : column.kind === "entier" || column.kind === "reel" ? (
                          <Hash size={12} className="tree-icon" />
                        ) : (
                          <Type size={12} className="tree-icon" />
                        )}
                        <span className="tree-label">{column.name}</span>
                        <span className="column-kind">{column.kind}</span>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="tool-window-footer">
        <span>{tables.length} table(s)</span>
        {hasWorkspace && <span className="hint">Double-clic pour ouvrir</span>}
      </div>
    </div>
  );
}
