# FakeSQL Studio

**Apprenez les bases de données en français.** FakeSQL Studio est un mini-IDE léger (Tauri 2 + React + Rust) autour d'un petit langage de bases de données 100 % français : les mots-clés sont des verbes (*afficher, inserer, modifier, effacer*) et vos tables restent de simples fichiers `.txt` — lisibles, portables, versionnables.

> 🌐 Site web : [docs/index.html](docs/index.html) — ou ouvrir le dossier `docs/` dans votre navigateur.

## ✨ Fonctionnalités

- Éditeur de requêtes avec coloration syntaxique, autocomplétion, validation en direct et diagnostics
- 5 types natifs : `entier`, `reel`, `booleen`, `caractere`, `chaine`
- Stockage en fichiers texte (`manager.txt` + un fichier par table), compatible avec les anciens fichiers
- Panneau d'exploration des tables (colonnes, types, nombre de lignes)
- Historique de requêtes, console d'exécution, export CSV
- Raccourcis clavier : `Ctrl+O` ouvrir, `Ctrl+N` nouvelle requête, `Ctrl+Entrée` exécuter, `Ctrl+W` fermer l'onglet

## 🚀 Télécharger

Installateurs Windows (64 bits) — voir [Releases](https://github.com/youssef-ajmi/FakeSQL/releases) :

- `FakeSQL Studio_0.1.0_x64_en-US.msi` — installation classique (WiX)
- `FakeSQL Studio_0.1.0_x64-setup.exe` — installation NSIS

## 🗣️ Le langage en un coup d'œil

Les mots-clés ne sont pas sensibles à la casse. Commentaires avec `#` ou `//`. Plusieurs requêtes par ligne séparées par `;` (ou un retour à la ligne).

| Commande | Exemple |
|---|---|
| Créer une table | `creer fichier 'users' ( $id : entier , $nom : chaine )` |
| Insérer | `inserer dans 'users' : $nom = 'Ada' , $age = 36` |
| Afficher / filtrer | `afficher tous les 'users' si $age > 25` |
| Modifier | `modifier 'users' si $id = 1 : $nom = 'Alan'` |
| Effacer | `effacer de 'users' si $age >= 60` |
| Supprimer une table | `supprimer fichier 'users'` |
| Renommer une table | `renommer fichier 'users' en 'clients'` |

Opérateurs : `=`, `<>`, `!=`, `<`, `<=`, `>`, `>=`, combinables avec `et` / `ou`.
Chaînes entre apostrophes ; pour mettre une apostrophe dans une donnée, la doubler (`''`).
`booleen` s'écrit `vrai` / `faux` (sans guillemets).

## 📄 Exemples de valeurs

| entier | reel | booleen | caractere | chaine |
|--------|------|---------|-----------|--------|
| `12` | `3.5` | `vrai` / `faux` | `'A'` (1 caractère) | `'Bonjour'` |

## 🔨 Compiler depuis les sources

Prérequis : [Rust (stable)](https://rustup.rs) ≥ 1.77 et [Node.js](https://nodejs.org) ≥ 20. Sur Windows : Rust LTS MSVC et Visual Studio Build Tools.

```bash
npm install

# Développement (rechargement à chaud)
npm run tauri dev

# Tests du moteur (Rust)
cd src-tauri && cargo test

# Build release (exe + installateurs MSI et NSIS)
npm run tauri build
```

## 📁 Structure

```
src/            Interface React (éditeur CodeMirror, panneaux, dialogs)
src-tauri/      Moteur Rust (lexer, parser, exécuteur, stockage) + app Tauri
docs/           Site web du projet
```

---

© 2026 FakeSQL · Compilé avec [Rust](https://www.rust-lang.org) + [Tauri](https://tauri.app)
