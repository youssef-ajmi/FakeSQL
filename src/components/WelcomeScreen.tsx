import { Database, FolderOpen } from "lucide-react";

interface WelcomeScreenProps {
  onOpenFolder: () => void;
}

const EXAMPLES = [
  "creer fichier 'users' ( $id : entier , $name : chaine )",
  "inserer dans 'users' : $name = 'Ada'",
  "afficher tous les 'users' si $id = 1",
  "modifier 'users' si $id = 1 : $name = 'Alan'",
  "effacer de 'users' si $id = 1",
  "# plus : et / ou, <, >, <=, >=, <> ; plusieurs requêtes par ligne ou séparées par ;",
];

export function WelcomeScreen({ onOpenFolder }: WelcomeScreenProps) {
  return (
    <div className="welcome-screen">
      <div className="welcome-logo">
        <Database size={34} />
      </div>
      <h1>FakeSQL Studio</h1>
      <p className="welcome-subtitle">
        Écrivez, validez et exécutez vos requêtes FakeSQL dans un petit IDE inspiré de JetBrains.
      </p>

      <button type="button" className="primary-button large" onClick={onOpenFolder}>
        <FolderOpen size={16} /> Ouvrir un dossier workspace
      </button>
      <p className="welcome-hint">
        Le dossier contient <code>manager.txt</code> et les fichiers de tables. Il sera créé
        automatiquement au besoin.
      </p>

      <div className="welcome-examples">
        <h2>Commandes FakeSQL</h2>
        {EXAMPLES.map((example) => (
          <code key={example}>{example}</code>
        ))}
      </div>
    </div>
  );
}
