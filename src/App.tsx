import { useEffect, useState } from "react";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { MenuBar } from "./components/MenuBar";
import { StatusBar } from "./components/StatusBar";
import { Toolbar } from "./components/Toolbar";
import { WelcomeScreen } from "./components/WelcomeScreen";
import { AboutDialog } from "./components/dialogs/AboutDialog";
import { CreateTableDialog } from "./components/dialogs/CreateTableDialog";
import { SyntaxDialog } from "./components/dialogs/SyntaxDialog";
import { EditorTabs } from "./components/editor/EditorTabs";
import { QueryEditor } from "./components/editor/QueryEditor";
import { TablesPanel } from "./components/explorer/TablesPanel";
import { BottomPanel } from "./components/panels/BottomPanel";
import { HistoryPanel } from "./components/panels/HistoryPanel";
import { useAppStore } from "./state/useAppStore";

export default function App() {
  const store = useAppStore();
  const [showCreate, setShowCreate] = useState(false);
  const [showAbout, setShowAbout] = useState(false);
  const [showSyntax, setShowSyntax] = useState(false);
  const blocked = showCreate || showAbout || showSyntax;

  const {
    workspace,
    tables,
    tabs,
    activeTab,
    activeResults,
    activeResult,
    running,
    consoleEntries,
    history,
    bottomTab,
    setBottomTab,
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
  } = store;

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      const key = event.key.toLowerCase();

      if (event.key === "Enter") {
        // Never fire queries while a modal is open.
        event.preventDefault();
        if (!blocked) void runTab();
      } else if (key === "o") {
        event.preventDefault();
        void openFolder();
      } else if (key === "n") {
        event.preventDefault();
        newTab();
      } else if (key === "w") {
        event.preventDefault();
        if (activeTab) closeTab(activeTab.id);
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [runTab, openFolder, newTab, closeTab, activeTab, blocked]);

  const openTable = (name: string) => {
    void runText(`afficher tous les '${name}'`);
  };

  return (
    <div className="app-shell">
      <MenuBar
        hasWorkspace={Boolean(workspace)}
        onOpenFolder={() => void openFolder()}
        onNewQuery={() => newTab()}
        onCreateTable={() => setShowCreate(true)}
        onRun={() => void runTab()}
        onClearEditor={() => {
          if (activeTab) updateQuery(activeTab.id, "");
        }}
        onRefresh={() => void refreshTables()}
        onAbout={() => setShowAbout(true)}
        onSyntax={() => setShowSyntax(true)}
      />

      <Toolbar
        workspace={workspace}
        running={running}
        onOpenFolder={() => void openFolder()}
        onNewQuery={() => newTab()}
        onRun={() => void runTab()}
        onRefresh={() => void refreshTables()}
        onCreateTable={() => setShowCreate(true)}
      />

      <div className="workbench">
        <PanelGroup
          direction="horizontal"
          autoSaveId="fakesql-horizontal"
          className="workbench-panels"
        >
          <Panel defaultSize={20} minSize={12} className="side-panel">
            <TablesPanel
              tables={tables}
              hasWorkspace={Boolean(workspace)}
              onRefresh={() => void refreshTables()}
              onOpenFolder={() => void openFolder()}
              onOpenTable={openTable}
              onDeleteTable={(name) => void removeTable(name)}
              onCreateTable={() => setShowCreate(true)}
            />
          </Panel>

          <PanelResizeHandle className="resize-handle vertical" />

          <Panel minSize={35}>
            <PanelGroup
              direction="vertical"
              autoSaveId="fakesql-center"
              className="center-panels"
            >
              <Panel defaultSize={64} minSize={25} className="editor-area">
                {workspace ? (
                  <>
                    <EditorTabs
                      tabs={tabs}
                      activeTabId={activeTab?.id ?? ""}
                      onSelect={store.setActiveTabId}
                      onClose={closeTab}
                      onNew={() => newTab()}
                    />
                    {activeTab && (
                      <QueryEditor
                        key={activeTab.id}
                        query={activeTab.query}
                        tables={tables}
                        inactive={blocked}
                        onChange={(query) => updateQuery(activeTab.id, query)}
                        onRun={() => void runTab(activeTab.id)}
                        onCursor={(position) => store.setCursor(position)}
                      />
                    )}
                  </>
                ) : (
                  <WelcomeScreen onOpenFolder={() => void openFolder()} />
                )}
              </Panel>

              <PanelResizeHandle className="resize-handle horizontal" />

              <Panel defaultSize={36} minSize={15} className="bottom-panel">
                <BottomPanel
                  tab={bottomTab}
                  onChange={setBottomTab}
                  results={activeResults}
                  entries={consoleEntries}
                  running={running}
                  onClearConsole={clearConsole}
                />
              </Panel>
            </PanelGroup>
          </Panel>

          <PanelResizeHandle className="resize-handle vertical" />

          <Panel defaultSize={20} minSize={12} className="side-panel right-panel">
            <HistoryPanel
              history={history}
              onOpen={(query) => newTab(query)}
              onRerun={(query) => void runText(query)}
              onClear={clearHistory}
            />
          </Panel>
        </PanelGroup>
      </div>

      <StatusBar
        workspace={workspace}
        tableCount={tables.length}
        result={activeResult}
        running={running}
        cursor={store.cursor}
      />

      {showCreate && (
        <CreateTableDialog onClose={() => setShowCreate(false)} onSubmit={addTable} />
      )}
      {showAbout && <AboutDialog onClose={() => setShowAbout(false)} />}
      {showSyntax && <SyntaxDialog onClose={() => setShowSyntax(false)} />}
    </div>
  );
}
