import { invoke } from "@tauri-apps/api/core";
import { ask, open, save } from "@tauri-apps/plugin-dialog";

import type {
  CreateColumnInput,
  QueryResult,
  TableInfo,
  ValidationResult,
} from "./types";

export const api = {
  openWorkspace: (path: string) => invoke<string>("open_workspace", { path }),
  getWorkspace: () => invoke<string | null>("get_workspace"),
  listTables: () => invoke<TableInfo[]>("list_tables"),
  /** Runs an editor document: one result per executed statement. */
  runQuery: (query: string) => invoke<QueryResult[]>("run_query", { query }),
  validateQuery: (query: string) =>
    invoke<ValidationResult>("validate_query", { query }),
  workspaceIssues: () => invoke<string[]>("workspace_issues"),
  createTable: (name: string, columns: CreateColumnInput[]) =>
    invoke<TableInfo>("create_table", { name, columns }),
  deleteTable: (name: string) => invoke<string>("delete_table", { name }),
  /** Asks for a destination then writes the CSV through the Rust side. */
  exportCsv: async (fileName: string, content: string) => {
    const path = await save({
      title: "Exporter en CSV",
      defaultPath: fileName,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return null;
    return invoke<string>("export_csv", { path, content });
  },
  pickFolder: () =>
    open({
      directory: true,
      multiple: false,
      title: "Ouvrir un dossier workspace",
    }),
  confirm: (message: string, title: string) =>
    ask(message, { title, kind: "warning" }),
};

export function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return JSON.stringify(error);
}
