import { linter } from "@codemirror/lint";

import { api, isTauriRuntime } from "../lib/tauri";

export const fakeSqlLinter = linter(
  async (view) => {
    if (!isTauriRuntime()) return [];
    const text = view.state.doc.toString();
    if (!text.trim()) return [];

    try {
      const result = await api.validateQuery(text);
      return result.errors.map((error) => {
        const from = Math.min(Math.max(error.from, 0), text.length);
        const to = Math.min(
          Math.max(error.to, from + 1),
          text.length,
        );
        return {
          from,
          to,
          severity: "error" as const,
          message: error.message,
        };
      });
    } catch {
      return [];
    }
  },
  { delay: 600 },
);
