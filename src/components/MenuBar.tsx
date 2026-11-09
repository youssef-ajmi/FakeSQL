import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Database } from "lucide-react";

interface MenuItem {
  label?: string;
  shortcut?: string;
  action?: () => void;
  separator?: boolean;
  disabled?: boolean;
}

interface MenuBarProps {
  hasWorkspace: boolean;
  onOpenFolder: () => void;
  onNewQuery: () => void;
  onCreateTable: () => void;
  onRun: () => void;
  onClearEditor: () => void;
  onRefresh: () => void;
  onAbout: () => void;
  onSyntax: () => void;
}

export function MenuBar({
  hasWorkspace,
  onOpenFolder,
  onNewQuery,
  onCreateTable,
  onRun,
  onClearEditor,
  onRefresh,
  onAbout,
  onSyntax,
}: MenuBarProps) {
  const [openMenu, setOpenMenu] = useState<string | null>(null);

  useEffect(() => {
    const close = () => setOpenMenu(null);
    window.addEventListener("click", close);
    return () => window.removeEventListener("click", close);
  }, []);

  const closeWindow = () => {
    void getCurrentWindow().close();
  };

  const menus: Record<string, MenuItem[]> = {
    File: [
      { label: "Ouvrir un dossier…", shortcut: "Ctrl+O", action: onOpenFolder },
      { label: "Nouvelle requête", shortcut: "Ctrl+N", action: onNewQuery },
      {
        label: "Créer une table…",
        action: onCreateTable,
        disabled: !hasWorkspace,
      },
      { separator: true },
      { label: "Fermer la fenêtre", action: closeWindow },
    ],
    Query: [
      { label: "Exécuter", shortcut: "Ctrl+Enter", action: onRun },
      { label: "Vider l'éditeur", action: onClearEditor },
    ],
    View: [{ label: "Rafraîchir les tables", action: onRefresh, disabled: !hasWorkspace }],
    Help: [
      { label: "Syntaxe FakeSQL…", action: onSyntax },
      { label: "À propos de FakeSQL Studio", action: onAbout },
    ],
  };

  return (
    <div className="menu-bar" onClick={(event) => event.stopPropagation()}>
      <div className="brand">
        <span className="brand-mark">
          <Database size={14} />
        </span>
        FakeSQL Studio
      </div>
      <nav className="menus">
        {Object.entries(menus).map(([name, items]) => (
          <div className="menu" key={name}>
            <button
              type="button"
              className={`menu-trigger ${openMenu === name ? "open" : ""}`}
              onClick={() => setOpenMenu(openMenu === name ? null : name)}
            >
              {name}
            </button>
            {openMenu === name && (
              <div className="menu-dropdown">
                {items.map((item, index) =>
                  item.separator ? (
                    <div className="menu-separator" key={`sep-${index}`} />
                  ) : (
                    <button
                      type="button"
                      key={item.label}
                      className="menu-item"
                      disabled={item.disabled}
                      onClick={() => {
                        setOpenMenu(null);
                        item.action?.();
                      }}
                    >
                      <span>{item.label}</span>
                      {item.shortcut && <kbd>{item.shortcut}</kbd>}
                    </button>
                  ),
                )}
              </div>
            )}
          </div>
        ))}
      </nav>
    </div>
  );
}
