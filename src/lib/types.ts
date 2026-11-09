export interface Column {
  name: string;
  kind: string;
}

export interface TableInfo {
  name: string;
  columns: Column[];
  rowCount: number;
}

export interface QueryResult {
  command: string;
  success: boolean;
  message: string | null;
  columns: string[] | null;
  rows: string[][] | null;
  rowCount: number | null;
  affected: number | null;
  elapsedMs: number;
}

export interface Diagnostic {
  message: string;
  from: number;
  to: number;
  severity: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: Diagnostic[];
}

export interface QueryTab {
  id: string;
  title: string;
  query: string;
}

export type ConsoleKind = "command" | "info" | "success" | "error";

export interface ConsoleEntry {
  id: string;
  time: string;
  kind: ConsoleKind;
  text: string;
}

export interface HistoryEntry {
  id: string;
  time: string;
  query: string;
  success: boolean;
  message: string;
}

export interface CreateColumnInput {
  name: string;
  kind: "entier" | "reel" | "booleen" | "caractere" | "chaine";
}

export type BottomTab = "results" | "console";
