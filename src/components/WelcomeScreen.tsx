import { useEffect, useState } from "react";
import { Check, Copy, Database, FolderOpen } from "lucide-react";

interface WelcomeScreenProps {
  onOpenFolder: () => void;
}

const TYPES: { name: string; example: string; description: string }[] = [
  { name: "entier", example: "12", description: "nombre entier" },
  { name: "reel", example: "3.5", description: "nombre à point décimal" },
  { name: "booleen", example: "vrai | faux", description: "sans guillemets" },
  { name: "caractere", example: "'A'", description: "un seul caractère" },
  { name: "chaine", example: "'texte'", description: "texte libre" },
];

const SHORTCUTS: [string, string][] = [
  ["Ctrl + O", "Ouvrir un dossier workspace"],
  ["Ctrl + N", "Nouvelle requête"],
  ["Ctrl + Enter", "Exécuter"],
  ["Ctrl + W", "Fermer l'onglet"],
];

const EXAMPLES: { label: string; query: string }[] = [
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

export function WelcomeScreen({ onOpenFolder }: WelcomeScreenProps) {
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
      // Clipboard unavailable (browser without permission): ignore.
    }
  };

  return (
    <div className="welcome-screen">
      <div className="welcome-logo">
        <Database size={34} />
      </div>
      <h1>FakeSQL Studio</h1>
      <p className="welcome-subtitle">
        Écrivez, validez et exécutez vos requêtes FakeSQL dans un petit IDE inspiré de
        JetBrains.
      </p>

      <button type="button" className="primary-button large" onClick={onOpenFolder}>
        <FolderOpen size={16} /> Ouvrir un dossier workspace
      </button>
      <p className="welcome-hint">
        Le dossier contient <code>manager.txt</code> et les fichiers de tables. Il sera créé
        automatiquement au besoin.
      </p>

      <div className="welcome-types">
        {TYPES.map((type) => (
          <div className="welcome-type" key={type.name}>
            <code className="type-name">{type.name}</code>
            <code className="type-example">{type.example}</code>
            <span>{type.description}</span>
          </div>
        ))}
      </div>

      <div className="welcome-panel">
        <div className="welcome-panel-head">
          <h2>Commandes FakeSQL</h2>
          <span className="welcome-hint-inline">cliquez pour copier</span>
        </div>
        {EXAMPLES.map((example, index) => (
          <button
            type="button"
            className={`welcome-example ${copiedIndex === index ? "copied" : ""}`}
            key={example.query}
            title="Copier dans le presse-papiers"
            onClick={() => void copyExample(index, example.query)}
          >
            <span className="welcome-example-label">{example.label}</span>
            <code>
              {copiedIndex === index ? (
                <span className="copied-mark">
                  <Check size={11} /> Copié
                </span>
              ) : (
                <span className="copy-mark">
                  <Copy size={11} /> Copier
                </span>
              )}
              {example.query}
            </code>
          </button>
        ))}
      </div>

      <div className="welcome-shortcuts">
        {SHORTCUTS.map(([keys, label]) => (
          <span className="welcome-shortcut" key={keys}>
            <kbd>{keys}</kbd>
            <span>{label}</span>
          </span>
        ))}
      </div>
    </div>
  );
}
