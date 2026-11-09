import { useEffect, useState } from "react";
import {
  Check,
  ChevronRight,
  Copy,
  Database,
  FolderOpen,
  Layers,
  BookOpen,
  Play,
} from "lucide-react";

interface WelcomeScreenProps {
  onOpenFolder: () => void;
  onSyntax?: () => void;
}

const VERSION = "v0.1.0";

const MEMO: { label: string; query: string }[] = [
  {
    label: "créer une table avec les 5 types",
    query:
      "creer fichier 'users' ( $id : entier , $prix : reel , $actif : booleen , $initiale : caractere , $nom : chaine )",
  },
  {
    label: "insérer une ligne (l'id est auto-incrémenté)",
    query: "inserer dans 'users' : $prix = 9.99 , $actif = vrai , $initiale = 'A' , $nom = 'Ada'",
  },
  {
    label: "afficher et filtrer",
    query: "afficher tous les 'users' si $prix >= 10 et $actif = vrai",
  },
  {
    label: "mettre à jour / effacer",
    query:
      "modifier 'users' si $id = 1 : $nom = 'Alan'\neffacer de 'users' si $id = 2",
  },
];

const TYPES: { name: string; example: string }[] = [
  { name: "entier", example: "12" },
  { name: "reel", example: "3.5" },
  { name: "booleen", example: "vrai | faux" },
  { name: "caractere", example: "'A'" },
  { name: "chaine", example: "'texte'" },
];

const STEPS: { label: string; icon: "folder" | "layers" | "db" | "play" }[] = [
  { label: "Ouvrir un dossier", icon: "folder" },
  { label: "Créer une table", icon: "layers" },
  { label: "Insérer vos données", icon: "db" },
  { label: "Exécuter", icon: "play" },
];

const SHORTCUTS: [string, string][] = [
  ["Ctrl + O", "Ouvrir un dossier"],
  ["Ctrl + N", "Nouvelle requête"],
  ["Ctrl + Enter", "Exécuter"],
  ["Ctrl + W", "Fermer l'onglet"],
];

function StepIcon({ kind }: { kind: (typeof STEPS)[number]["icon"] }) {
  const size = 13;
  if (kind === "folder") return <FolderOpen size={size} />;
  if (kind === "layers") return <Layers size={size} />;
  if (kind === "play") return <Play size={size} fill="currentColor" />;
  return <Database size={size} />;
}

export function WelcomeScreen({ onOpenFolder, onSyntax }: WelcomeScreenProps) {
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  useEffect(() => {
    if (copiedIndex == null) return;
    const timer = setTimeout(() => setCopiedIndex(null), 1400);
    return () => clearTimeout(timer);
  }, [copiedIndex]);

  const copyExample = async (index: number, query: string) => {
    try {
      await navigator.clipboard.writeText(query);
      setCopiedIndex(index);
    } catch {
      // Clipboard unavailable: ignore.
    }
  };

  return (
    <div className="welcome-screen">
      <div className="welcome-inner">
        <div className="welcome-top">
          <span className="welcome-brandmark">
            <Database size={16} />
          </span>
          <span className="welcome-version">{VERSION}</span>
        </div>

        <div className="welcome-hero">
          <h1>
            L'éditeur <span className="hl">FakeSQL</span>
          </h1>
          <p>
            Écrivez, validez et exécutez vos requêtes dans un mini-IDE léger. Vos tables
            restent de simples fichiers <code>.txt</code> — lisibles, portables, versionnables.
          </p>
        </div>

        <div className="welcome-cta-row">
          <button type="button" className="primary-button large" onClick={onOpenFolder}>
            <FolderOpen size={16} /> Ouvrir un dossier workspace
          </button>
          {onSyntax && (
            <button type="button" className="secondary-button large" onClick={onSyntax}>
              <BookOpen size={15} /> Syntaxe complète
            </button>
          )}
        </div>

        <div className="welcome-steps">
          {STEPS.map((step, index) => (
            <div className="welcome-step" key={step.label}>
              {index === STEPS.length - 1 && (
                <ChevronRight size={14} className="step-arrow" />
              )}
              <div className="welcome-step-chip">
                <span className="step-num">{index + 1}</span>
                <StepIcon kind={step.icon} />
                <span className="step-label">
                  {step.label}
                </span>
              </div>
              {index < STEPS.length - 1 && (
                <ChevronRight size={14} className="step-arrow" />
              )}
            </div>
          ))}
        </div>

        <div className="welcome-cols">
          <section className="welcome-column">
            <div className="welcome-block-head">
              <h2>Aide-mémoire</h2>
              <span className="welcome-head-hint">cliquez pour copier</span>
            </div>
            <div className="welcome-terminal">
              <div className="terminal-bar">
                <span className="terminal-dot red" />
                <span className="terminal-dot yellow" />
                <span className="terminal-dot green" />
                <span className="terminal-file">requêtes.fsql</span>
              </div>
              {MEMO.map((item, index) => (
                <button
                  type="button"
                  className={`terminal-row ${copiedIndex === index ? "copied" : ""}`}
                  key={item.label}
                  title={item.label}
                  onClick={() => void copyExample(index, item.query)}
                >
                  <span className="terminal-label">{item.label}</span>
                  <code className="terminal-cmd">{item.query}</code>
                  <span className="copy-chip">
                    {copiedIndex === index ? (
                      <>
                        <Check size={11} /> Copié
                      </>
                    ) : (
                      <>
                        <Copy size={11} /> Copier
                      </>
                    )}
                  </span>
                </button>
              ))}
            </div>
          </section>

          <section className="welcome-column">
            <div className="welcome-block-head">
              <h2>Les 5 types</h2>
            </div>
            <div className="welcome-types">
              {TYPES.map((type) => (
                <div className="welcome-type" key={type.name}>
                  <code className="type-name">{type.name}</code>
                  <code className="type-example">{type.example}</code>
                </div>
              ))}
            </div>
            <div className="welcome-note">
              <span>
                Le workspace contient <code>manager.txt</code> et les fichiers de tables,
                créés automatiquement au besoin. Formats booléen : <code>vrai</code> /{" "}
                <code>faux</code> (sans guillemets) ; apostrophes doublées :{" "}
                <code>''</code>.
              </span>
            </div>
            <div className="welcome-footer">
              {SHORTCUTS.map(([keys, label]) => (
                <span className="welcome-shortcut" key={keys}>
                  <kbd>{keys}</kbd>
                  <span>{label}</span>
                </span>
              ))}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
