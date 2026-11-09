import { useCallback, useEffect, useRef, useState } from "react";

import { api, errorMessage, isTauriRuntime } from "../lib/tauri";
import type {
  BottomTab,
  ConsoleEntry,
  ConsoleKind,
  CreateColumnInput,
  HistoryEntry,
  QueryResult,
  QueryTab,
  TableInfo,
} from "../lib/types";

const WORKSPACE_KEY = "fakesql.workspace";
const TABS_KEY = "fakesql.tabs";
const HISTORY_KEY = "fakesql.history";

function uid() {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : Math.random().toString(36).slice(2);
}

function clock() {
  return new Date().toLocaleTimeString("fr-FR", { hour12: false });
}

function makeTab(query = "", title = "Query 1"): QueryTab {
  return { id: uid(), title, query };
}

interface PersistedTabs {
  tabs: QueryTab[];
  activeTabId: string;
}

function loadTabs(): PersistedTabs {
  try {
    const raw = localStorage.getItem(TABS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as PersistedTabs;
      if (Array.isArray(parsed.tabs) && parsed.tabs.length > 0) {
        const activeTabId = parsed.tabs.some((tab) => tab.id === parsed.activeTabId)
          ? parsed.activeTabId
          : parsed.tabs[0].id;
        return { tabs: parsed.tabs, activeTabId };
      }
    }
  } catch {
    // ignore malformed storage
  }
  const tab = makeTab();
  return { tabs: [tab], activeTabId: tab.id };
}

function loadHistory(): HistoryEntry[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as HistoryEntry[]) : [];
  } catch {
    return [];
  }
}

function nextTabNumber(tabs: QueryTab[]) {
  return (
    tabs.reduce((max, tab) => {
      const match = /Query (\d+)/.exec(tab.title);
      return match ? Math.max(max, Number(match[1])) : max;
    }, 0) + 1
  );
}

export function useAppStore() {
  const initialRef = useRef<PersistedTabs | null>(null);
  if (!initialRef.current) {
    initialRef.current = loadTabs();
  }

  const [tabs, setTabs] = useState<QueryTab[]>(initialRef.current.tabs);
  const [activeTabId, setActiveTabId] = useState(initialRef.current.activeTabId);
  const [workspace, setWorkspace] = useState<string | null>(null);
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [results, setResults] = useState<Record<string, QueryResult[] | null>>({});
  const [running, setRunning] = useState(false);
  const [consoleEntries, setConsoleEntries] = useState<ConsoleEntry[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>(loadHistory);
  const [bottomTab, setBottomTab] = useState<BottomTab>("console");
  const [cursor, setCursor] = useState<{ line: number; column: number }>({
    line: 1,
    column: 1,
  });

  const addConsole = useCallback((kind: ConsoleKind, text: string) => {
    setConsoleEntries((previous) => [
      ...previous.slice(-299),
      { id: uid(), time: clock(), kind, text },
    ]);
  }, []);

  const addHistory = useCallback(
    (query: string, success: boolean, message: string, elapsedMs: number) => {
      setHistory((previous) => {
        // Collapse consecutive identical runs instead of flooding the list.
        if (previous[0]?.query === query) {
          return previous;
        }
        const next = [
          { id: uid(), time: clock(), query, success, message, elapsedMs },
          ...previous,
        ].slice(0, 100);
        localStorage.setItem(HISTORY_KEY, JSON.stringify(next));
        return next;
      });
    },
    [],
  );

  const refreshTables = useCallback(async () => {
    try {
      const list = await api.listTables();
      setTables(list);
      try {
        const issues = await api.workspaceIssues();
        for (const issue of issues) {
          addConsole("info", `Attention : ${issue}`);
        }
      } catch {
        // Issues are best-effort; do not spam the console on failure.
      }
      return list;
    } catch (error) {
      const message = errorMessage(error);
      if (!message.includes("Aucun dossier")) {
        addConsole("error", message);
      }
      setTables([]);
      return [];
    }
  }, [addConsole]);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    const saved = localStorage.getItem(WORKSPACE_KEY);
    if (!saved) return;
    let cancelled = false;

    void (async () => {
      try {
        const root = await api.openWorkspace(saved);
        if (cancelled) return;
        setWorkspace(root);
        const list = await api.listTables();
        if (!cancelled) setTables(list);
      } catch {
        localStorage.removeItem(WORKSPACE_KEY);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  // Tables can be edited outside the app; refresh on window focus.
  useEffect(() => {
    if (!isTauriRuntime() || !workspace) return;
    const onFocus = () => void refreshTables();
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [workspace, refreshTables]);

  useEffect(() => {
    localStorage.setItem(TABS_KEY, JSON.stringify({ tabs, activeTabId }));
  }, [tabs, activeTabId]);

  const openFolder = useCallback(async () => {
    if (!isTauriRuntime()) {
      addConsole("error", "Le workspace nécessite l'application Tauri.");
      return;
    }
    try {
      const selected = await api.pickFolder();
      if (typeof selected !== "string") return;
      const root = await api.openWorkspace(selected);
      setWorkspace(root);
      localStorage.setItem(WORKSPACE_KEY, root);
      const list = await api.listTables();
      setTables(list);
      addConsole("info", `Workspace ouvert : ${root}`);
      setBottomTab("console");
    } catch (error) {
      addConsole("error", errorMessage(error));
    }
  }, [addConsole]);

  const newTab = useCallback(
    (query = "") => {
      const tab = makeTab(query, `Query ${nextTabNumber(tabs)}`);
      setTabs((previous) => [...previous, tab]);
      setActiveTabId(tab.id);
      return tab;
    },
    [tabs],
  );

  const closeTab = useCallback(
    (id: string) => {
      const index = tabs.findIndex((tab) => tab.id === id);
      if (index === -1) return;

      const next = tabs.filter((tab) => tab.id !== id);
      if (next.length === 0) {
        const fresh = makeTab();
        setTabs([fresh]);
        setActiveTabId(fresh.id);
      } else {
        setTabs(next);
        if (activeTabId === id) {
          setActiveTabId(next[Math.max(0, index - 1)].id);
        }
      }

      setResults((previous) => {
        const copy = { ...previous };
        delete copy[id];
        return copy;
      });
    },
    [tabs, activeTabId],
  );

  const updateQuery = useCallback((id: string, query: string) => {
    setTabs((previous) =>
      previous.map((tab) => (tab.id === id ? { ...tab, query } : tab)),
    );
  }, []);

  const execute = useCallback(
    async (target: QueryTab) => {
      if (running) return;
      if (!workspace) {
        addConsole(
          "error",
          "Ouvrez un dossier workspace avant d'exécuter une requête.",
        );
        setBottomTab("console");
        return;
      }
      if (!target.query.trim()) {
        addConsole("info", "La requête est vide.");
        setBottomTab("console");
        return;
      }

      setRunning(true);
      addConsole("command", target.query);
      try {
        const executed = await api.runQuery(target.query);
        setResults((previous) => ({ ...previous, [target.id]: executed }));

        const count = executed.length;
        if (count === 0) {
          addConsole("info", "Aucune requête à exécuter.");
          setBottomTab("console");
        } else {
          for (const [index, result] of executed.entries()) {
            const prefix = count > 1 ? `#${index + 1} · ` : "";
            addConsole(
              result.success ? "success" : "error",
              `${prefix}${result.message ?? ""} · ${result.elapsedMs} ms`,
            );
          }
          const allSuccess = executed.every((result) => result.success);
          addHistory(
            target.query,
            allSuccess,
            executed[executed.length - 1].message ?? "",
            executed.reduce((total, result) => total + result.elapsedMs, 0),
          );
          // The results pane shows message cards for failed statements too.
          setBottomTab("results");
          if (allSuccess) {
            await refreshTables();
          }
        }
      } catch (error) {
        addConsole("error", errorMessage(error));
        setBottomTab("console");
      } finally {
        setRunning(false);
      }
    },
    [addConsole, addHistory, refreshTables, running, workspace],
  );

  const runTab = useCallback(
    async (tabId?: string) => {
      const target = tabs.find((tab) => tab.id === (tabId ?? activeTabId));
      if (target) await execute(target);
    },
    [tabs, activeTabId, execute],
  );

  const runText = useCallback(
    async (query: string) => {
      // Reuse an existing tab holding the same text instead of stacking tabs.
      const existing = tabs.find((tab) => tab.query === query);
      if (existing) {
        setActiveTabId(existing.id);
        await execute(existing);
        return;
      }
      const tab = makeTab(query, `Query ${nextTabNumber(tabs)}`);
      setTabs((previous) => [...previous, tab]);
      setActiveTabId(tab.id);
      await execute(tab);
    },
    [tabs, execute],
  );

  const addTable = useCallback(
    async (name: string, columns: CreateColumnInput[]) => {
      const info = await api.createTable(name, columns);
      addConsole("success", `Table '${info.name}' créée`);
      await refreshTables();
      return info;
    },
    [addConsole, refreshTables],
  );

  const removeTable = useCallback(
    async (name: string) => {
      const confirmed = await api.confirm(
        `Supprimer la table '${name}' et son fichier de données ?`,
        "Supprimer la table",
      );
      if (!confirmed) return;
      try {
        const message = await api.deleteTable(name);
        addConsole("success", message);
        await refreshTables();
      } catch (error) {
        addConsole("error", errorMessage(error));
      }
    },
    [addConsole, refreshTables],
  );

  const clearConsole = useCallback(() => setConsoleEntries([]), []);
  const clearHistory = useCallback(() => {
    setHistory([]);
    localStorage.removeItem(HISTORY_KEY);
  }, []);

  const activeTab = tabs.find((tab) => tab.id === activeTabId) ?? tabs[0];
  const activeResults = activeTab ? results[activeTab.id] ?? null : null;
  const activeResult =
    activeResults && activeResults.length > 0
      ? activeResults[activeResults.length - 1]
      : null;

  // A newly opened tab's editor reports its own position; until then 1:1.
  useEffect(() => {
    setCursor({ line: 1, column: 1 });
  }, [activeTabId]);

  return {
    workspace,
    tables,
    tabs,
    activeTab,
    activeResults,
    activeResult,
    results,
    running,
    consoleEntries,
    history,
    bottomTab,
    cursor,
    setCursor,
    setBottomTab,
    setActiveTabId,
    openFolder,
    refreshTables,
    newTab,
    closeTab,
    updateQuery,
    runTab,
    runText,
    addTable,
    removeTable,
    clearConsole,
    clearHistory,
  };
}

export type AppStore = ReturnType<typeof useAppStore>;
