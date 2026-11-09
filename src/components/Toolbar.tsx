import { Database, FolderOpen, Plus, RefreshCw, SquarePlus } from "lucide-react";

interface ToolbarProps {
  workspace: string | null;
  onOpenFolder: () => void;
  onNewQuery: () => void;
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
  onOpenFolder,
  onNewQuery,
  onRefresh,
  onCreateTable,
}: ToolbarProps) {
  return (
    <div className="toolbar">
      <div className="toolbar-brand">
        <b>FakeSQL</b> Studio
      </div>

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
      </div>

      <div className="toolbar-separator" />

      <div className="toolbar-group">
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

      <div className="toolbar-spacer" />

      <button
        type="button"
        className="workspace-pill clickable"
        title={workspace ? "Changer de workspace (Ctrl+O)" : "Ouvrir un workspace (Ctrl+O)"}
        onClick={onOpenFolder}
      >
        <Database size={13} />
        <span>{workspaceLabel(workspace)}</span>
      </button>
    </div>
  );
}

