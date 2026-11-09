import type { Completion, CompletionContext, CompletionResult } from "@codemirror/autocomplete";

import type { TableInfo } from "../lib/types";
import { FAKESQL_KEYWORDS } from "./fakesqlLanguage";

export function fakeSqlCompletions(getTables: () => TableInfo[]) {
  return (context: CompletionContext): CompletionResult | null => {
    const variable = context.matchBefore(/\$[A-Za-z0-9_]*/);
    if (variable) {
      const columns = new Map<string, Completion>();
      for (const table of getTables()) {
        for (const column of table.columns) {
          if (!columns.has(column.name)) {
            columns.set(column.name, {
              label: `$${column.name}`,
              type: "property",
              detail: column.kind,
            });
          }
        }
      }
      return {
        from: variable.from,
        options: [...columns.values()],
        validFor: /^\$[A-Za-z0-9_]*$/,
      };
    }

    const quoted = context.matchBefore(/'[^']*'?/);
    if (quoted) {
      return {
        from: quoted.from,
        options: getTables().map((table) => ({
          label: `'${table.name}'`,
          apply: `'${table.name}'`,
          type: "variable",
          detail: `${table.rowCount} ligne(s)`,
        })),
      };
    }

    const word = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
    if (!context.explicit && !word) return null;

    const options: Completion[] = [
      ...FAKESQL_KEYWORDS.map((keyword) => ({
        label: keyword,
        type: "keyword",
      })),
      ...getTables().map((table) => ({
        label: `'${table.name}'`,
        apply: `'${table.name}'`,
        type: "variable",
        detail: `${table.rowCount} ligne(s)`,
      })),
    ];

    return {
      from: word ? word.from : context.pos,
      options,
      validFor: /^[A-Za-z_][A-Za-z0-9_]*$/,
    };
  };
}
