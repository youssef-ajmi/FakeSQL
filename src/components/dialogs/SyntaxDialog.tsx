import { BookOpen, X } from "lucide-react";

interface SyntaxDialogProps {
  onClose: () => void;
}

interface SyntaxSection {
  code: string;
  description?: string;
  note?: string;
}

interface SyntaxCommand {
  name: string;
  title: string;
  sections: SyntaxSection[];
}

const COMMANDS: SyntaxCommand[] = [
  {
    name: "afficher",
    title: "Afficher les données",
    sections: [
      {
        code: "afficher tous les '<table>'",
      },
      {
        code: "afficher tous les '<table>' si <$colonne> <opérateur> <valeur>",
        description:
          "Filtre optionnel : la requête utilise la même condition que effacer et modifier.",
        note: "ex. afficher tous les 'users' si $age > 25",
      },
    ],
  },
  {
    name: "inserer",
    title: "Insérer une ligne",
    sections: [
      {
        code: "inserer dans '<table>' : $colonne = <valeur> [ , $colonne = <valeur> ]",
        note: "ex. inserer dans 'users' : $name = 'Ada' , $password = '1234'",
      },
      {
        code: "Règle : la colonne 'id' de type entier est auto-incrémentée si elle n'est pas fournie. Les autres colonnes sont obligatoires.",
      },
    ],
  },
  {
    name: "modifier",
    title: "Modifier des lignes",
    sections: [
      {
        code: "modifier '<table>' si <$colonne> <opérateur> <valeur> : $colonne = <valeur> [ , ... ]",
        note: "ex. modifier 'users' si $id = 1 : $name = 'Alan'",
      },
      {
        code: "Règle : impossible de changer 'id' si la condition cible plusieurs lignes, et la nouvelle valeur d'id doit rester unique.",
      },
    ],
  },
  {
    name: "effacer",
    title: "Effacer des lignes",
    sections: [
      {
        code: "effacer de '<table>' si <$colonne> <opérateur> <valeur> [et / ou ...]",
        note: "ex. effacer de 'users' si $age >= 60",
      },
    ],
  },
  {
    name: "creer",
    title: "Créer une table",
    sections: [
      {
        code: "creer fichier '<table>' ( $colonne : entier | reel | booleen | caractere | chaine [ , ... ] )",
        note: "ex. creer fichier 'users' ( $id : entier , $prix : reel , $actif : booleen )",
      },
      {
        code: "Règle : la déclaration peut s'écrire sur plusieurs lignes à l'intérieur des parenthèses.",
      },
    ],
  },
  {
    name: "supprimer / renommer",
    title: "Supprimer ou renommer une table",
    sections: [
      {
        code: "supprimer fichier '<table>'",
      },
      {
        code: "renommer fichier '<ancien>' en '<nouveau>'",
      },
    ],
  },
];

export function SyntaxDialog({ onClose }: SyntaxDialogProps) {
  return (
    <div className="modal-overlay" onMouseDown={onClose}>
      <div
        className="dialog syntax-dialog"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="dialog-header">
          <span className="dialog-title">
            <BookOpen size={15} /> Syntaxe FakeSQL
          </span>
          <button type="button" className="icon-button small" onClick={onClose} title="Fermer">
            <X size={14} />
          </button>
        </div>

        <div className="dialog-body syntax-body">
          {COMMANDS.map((command) => (
            <section className="syntax-section" key={command.name}>
              <h3>
                <span className="syntax-keyword">{command.name}</span> — {command.title}
              </h3>
              {command.sections.map((section, index) => (
                <div className="syntax-item" key={`${command.name}-${index}`}>
                  {section.description && <p>{section.description}</p>}
                  <code>{section.code}</code>
                  {section.note && <span className="syntax-note">{section.note}</span>}
                </div>
              ))}
            </section>
          ))}

          <section className="syntax-section">
            <h3>Généralités</h3>
            <ul className="syntax-rules">
              <li>
                <code>Types</code> : <code>entier</code> (12), <code>reel</code> (3.5, point
                décimal), <code>booleen</code> (<code>vrai</code> / <code>faux</code>, sans
                guillemets), <code>caractere</code> (un seul caractère, ex.{" "}
                <code>'A'</code>) et <code>chaine</code> (texte libre).
              </li>
              <li>
                <code>Valeurs</code> : le type de la valeur doit correspondre à celui de
                la colonne. Ce qui est entre guillemets est une chaîne, un nombre nu
                est un <code>entier</code> ou un <code>reel</code> (<code>10</code> vaut{" "}
                <code>10.0</code> pour un <code>reel</code>, mais un <code>entier</code>{" "}
                ne peut pas recevoir <code>3.5</code>).
              </li>
              <li>
                <code>Apostrophes</code> : pour les mettre dans une valeur, doublez-les
                (<code>'it''s'</code> donne <code>it's</code>). Les symboles{" "}
                <code>|</code> et <code>&gt;</code> sont stockés normalement.
              </li>
              <li>
                <code>Opérateurs</code> : <code>=</code>, <code>&lt;&gt;</code>,{" "}
                <code>!=</code>, <code>&lt;</code>, <code>&lt;=</code>,{" "}
                <code>&gt;</code>, <code>&gt;=</code>. Les conditions se combinent
                avec <code>et</code> / <code>ou</code> (lues de gauche à droite).
              </li>
              <li>
                <code>Commentaires</code> : <code>#</code> ou <code>//</code> jusqu'à
                la fin de la ligne.
              </li>
              <li>
                <code>Requêtes multiples</code> : séparez-les par <code>;</code> ou un
                retour à la ligne ; l'exécution s'arrête à la première erreur.
              </li>
              <li>
                <code>Noms</code> : table et colonnes acceptent lettres, chiffres,{" "}
                <code>_</code> et <code>-</code> (les colonnes s'écrivent avec{" "}
                <code>$</code>). Les mots-clés ne sont pas sensibles à la casse.
              </li>
            </ul>
          </section>
        </div>

        <div className="dialog-footer">
          <button type="button" className="primary-button" onClick={onClose}>
            Fermer
          </button>
        </div>
      </div>
    </div>
  );
}
