import {
  Database,
  FolderOpen,
  Loader2,
  Play,
  Plus,
  RefreshCw,
  SquarePlus,
} from "lucide-react";

interface ToolbarProps {
  workspace: string | null;
  running: boolean;
  onOpenFolder: () => void;
  onNewQuery: () => void;
  onRun: () => void;
  onRefresh: () => void;
  onCreateTable: () => void;
}

function workspaceLabel(workspace: string | null) {
  if (!workspace) return "Aucun workspace";
  const parts = workspace.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? workspace;
}

export function Toolbar({
  workspace,
  running,
  onOpenFolder,
  onNewQuery,
  onRun,
  onRefresh,
  onCreateTable,
}: ToolbarProps) {
  return (
    <div className="toolbar">
      <div className="toolbar-group">
        <button
          type="button"
          className="icon-button"
          title="Ouvrir un dossier (Ctrl+O)"
          onClick={onOpenFolder}
        >
          <FolderOpen size={17} />
        </button>
        <button
          type="button"
          className="icon-button"
          title="Nouvelle requête (Ctrl+N)"
          onClick={onNewQuery}
        >
          <SquarePlus size={17} />
        </button>
        <button
          type="button"
          className="icon-button"
          title="Créer une table"
          onClick={onCreateTable}
          disabled={!workspace}
        >
          <Plus size={17} />
        </button>
        <button
          type="button"
          className="icon-button"
          title="Rafraîchir les tables"
          onClick={onRefresh}
          disabled={!workspace}
        >
          <RefreshCw size={16} />
        </button>
      </div>

      <div className="toolbar-separator" />

      <button
        type="button"
        className="run-button"
        onClick={onRun}
        disabled={running || !workspace}
        title="Exécuter la requête (Ctrl+Enter)"
      >
        {running ? <Loader2 size={15} className="spin" /> : <Play size={14} fill="currentColor" />}
        <span>{running ? "Exécution…" : "Run"}</span>
      </button>

      <div className="toolbar-spacer" />

      <div className="workspace-pill" title={workspace ?? "Aucun dossier ouvert"}>
        <Database size={13} />
        <span>{workspaceLabel(workspace)}</span>
      </div>
    </div>
  );
}
