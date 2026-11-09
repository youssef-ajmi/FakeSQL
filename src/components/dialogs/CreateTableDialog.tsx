import { useEffect, useState } from "react";
import { Plus, Table2, Trash2, X } from "lucide-react";

import { errorMessage } from "../../lib/tauri";
import type { CreateColumnInput, TableInfo } from "../../lib/types";

interface CreateTableDialogProps {
  onClose: () => void;
  onSubmit: (name: string, columns: CreateColumnInput[]) => Promise<TableInfo>;
}

type ColumnKind = CreateColumnInput["kind"];

interface ColumnDraft {
  id: string;
  name: string;
  kind: ColumnKind;
}

function draft(name = "", kind: ColumnKind = "chaine"): ColumnDraft {
  return { id: crypto.randomUUID(), name, kind };
}

const COLUMN_KINDS: { value: ColumnKind; label: string }[] = [
  { value: "entier", label: "entier" },
  { value: "reel", label: "reel" },
  { value: "booleen", label: "booleen" },
  { value: "caractere", label: "caractere" },
  { value: "chaine", label: "chaine" },
];

export function CreateTableDialog({ onClose, onSubmit }: CreateTableDialogProps) {
  const [name, setName] = useState("");
  const [columns, setColumns] = useState<ColumnDraft[]>([draft("id", "entier")]);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  const updateColumn = (id: string, patch: Partial<ColumnDraft>) => {
    setColumns((previous) =>
      previous.map((column) => (column.id === id ? { ...column, ...patch } : column)),
    );
  };

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    const tableName = name.trim().replace(/^'|'$/g, "");
    if (!tableName) {
      setError("Le nom de la table est requis.");
      return;
    }

    const cleaned = columns
      .map((column) => ({
        name: column.name.trim().replace(/^\$/, ""),
        kind: column.kind,
      }))
      .filter((column) => column.name.length > 0);

    if (cleaned.length === 0) {
      setError("Ajoutez au moins une colonne.");
      return;
    }

    const duplicate = cleaned.find(
      (column, index) => cleaned.findIndex((candidate) => candidate.name === column.name) !== index,
    );
    if (duplicate) {
      setError(`La colonne '${duplicate.name}' est donnée deux fois.`);
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      await onSubmit(tableName, cleaned);
      onClose();
    } catch (submitError) {
      setError(errorMessage(submitError));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="modal-overlay" onMouseDown={onClose}>
      <form
        className="dialog create-table-dialog"
        onSubmit={submit}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="dialog-header">
          <span className="dialog-title">
            <Table2 size={15} /> Créer une table
          </span>
          <button type="button" className="icon-button small" onClick={onClose} title="Fermer">
            <X size={14} />
          </button>
        </div>

        <div className="dialog-body">
          <label className="field">
            <span>Nom de la table</span>
            <input
              autoFocus
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="users"
              spellCheck={false}
            />
          </label>

          <div className="field">
            <span>Colonnes</span>
            <div className="column-editor">
              {columns.map((column) => (
                <div className="column-row" key={column.id}>
                  <input
                    value={column.name}
                    onChange={(event) => updateColumn(column.id, { name: event.target.value })}
                    placeholder="nom"
                    spellCheck={false}
                  />
                  <select
                    value={column.kind}
                    onChange={(event) =>
                      updateColumn(column.id, {
                        kind: event.target.value as ColumnKind,
                      })
                    }
                  >
                    {COLUMN_KINDS.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                  <button
                    type="button"
                    className="icon-button small"
                    title="Supprimer la colonne"
                    onClick={() =>
                      setColumns((previous) => previous.filter((item) => item.id !== column.id))
                    }
                    disabled={columns.length === 1}
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              ))}
            </div>
            <button
              type="button"
              className="secondary-button add-column"
              onClick={() => setColumns((previous) => [...previous, draft()])}
            >
              <Plus size={13} /> Ajouter une colonne
            </button>
          </div>

          <p className="dialog-note">
            Une colonne <code>id</code> de type <code>entier</code> est auto-incrémentée si elle
            n'est pas fournie lors de l'insertion.
          </p>

          {error && <div className="dialog-error">{error}</div>}
        </div>

        <div className="dialog-footer">
          <button type="button" className="secondary-button" onClick={onClose}>
            Annuler
          </button>
          <button type="submit" className="primary-button" disabled={submitting}>
            {submitting ? "Création…" : "Créer la table"}
          </button>
        </div>
      </form>
    </div>
  );
}
