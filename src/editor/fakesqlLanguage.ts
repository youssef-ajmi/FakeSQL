import { StreamLanguage } from "@codemirror/language";

export const FAKESQL_KEYWORDS = [
  "afficher",
  "tous",
  "les",
  "inserer",
  "dans",
  "effacer",
  "de",
  "si",
  "creer",
  "fichier",
  "modifier",
  "supprimer",
  "renommer",
  "en",
  "et",
  "ou",
  "entier",
  "reel",
  "booleen",
  "caractere",
  "chaine",
  "vrai",
  "faux",
];

const keywordSet = new Set(FAKESQL_KEYWORDS);

export const fakeSqlLanguage = StreamLanguage.define({
  name: "fakesql",
  startState: () => ({}),
  token(stream) {
    if (stream.eatSpace()) return null;

    if (stream.match(/'[^']*(''[^']*)*'/)) return "string";
    if (stream.match(/\$[A-Za-z0-9_]+/)) return "variableName";
    if (stream.match(/\d+(\.\d+)?/)) return "number";
    if (stream.match(/[(),:=]/)) return "operator";

    if (stream.match(/[A-Za-z_][A-Za-z0-9_]*/)) {
      return keywordSet.has(stream.current().toLowerCase()) ? "keyword" : "variableName";
    }

    stream.next();
    return null;
  },
});
