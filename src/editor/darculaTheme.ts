import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags } from "@lezer/highlight";

const editorTheme = EditorView.theme(
  {
    "&": {
      color: "#bcbec4",
      backgroundColor: "#1e1f22",
      height: "100%",
      fontSize: "13px",
    },
    ".cm-scroller": {
      fontFamily:
        "'JetBrains Mono', 'Cascadia Code', 'SFMono-Regular', Consolas, monospace",
      lineHeight: "1.65",
      overflow: "auto",
    },
    ".cm-content": {
      caretColor: "#ffffff",
      padding: "12px 0 24px",
    },
    ".cm-line": {
      padding: "0 18px",
    },
    "&.cm-focused .cm-cursor": {
      borderLeftColor: "#ffffff",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
      {
        backgroundColor: "#214283",
      },
    ".cm-gutters": {
      backgroundColor: "#1e1f22",
      color: "#4b5059",
      border: "none",
      paddingLeft: "8px",
    },
    ".cm-activeLine": {
      backgroundColor: "#26282e",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "#26282e",
      color: "#a1a3a6",
    },
    ".cm-lineNumbers .cm-gutterElement": {
      padding: "0 10px 0 6px",
      minWidth: "34px",
    },
    ".cm-matchingBracket": {
      backgroundColor: "#3d5a80",
      outline: "1px solid #3574f0",
    },
    ".cm-tooltip": {
      backgroundColor: "#2b2d30",
      border: "1px solid #393b40",
      color: "#bcbec4",
      borderRadius: "4px",
      boxShadow: "0 8px 24px rgba(0, 0, 0, 0.45)",
    },
    ".cm-tooltip-autocomplete ul li[aria-selected]": {
      backgroundColor: "#2e436e",
      color: "#ffffff",
    },
    ".cm-tooltip-autocomplete ul li": {
      padding: "3px 8px",
    },
    ".cm-panels": {
      backgroundColor: "#2b2d30",
      color: "#bcbec4",
    },
    ".cm-searchMatch": {
      backgroundColor: "#32593d",
      outline: "1px solid #499c54",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "#2e436e",
    },
    ".cm-lintRange-error": {
      backgroundImage:
        "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='6' height='3'%3E%3Cpath d='M0 3 L2 0 L4 3 L6 0' fill='none' stroke='%23ff6b68' stroke-width='1'/%3E%3C/svg%3E\")",
      backgroundRepeat: "repeat-x",
      backgroundPosition: "bottom",
    },
    ".cm-tooltip-lint": {
      backgroundColor: "#2b2d30",
    },
  },
  { dark: true },
);

const highlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: "#cf8e6d", fontWeight: "600" },
  { tag: tags.string, color: "#6a8759" },
  { tag: tags.number, color: "#2aacb8" },
  { tag: tags.variableName, color: "#c77dbb" },
  { tag: tags.operator, color: "#bcbec4" },
  { tag: tags.punctuation, color: "#bcbec4" },
  { tag: tags.comment, color: "#7a7e85", fontStyle: "italic" },
  { tag: tags.invalid, color: "#ff6b68" },
]);

export const darculaTheme = [editorTheme, syntaxHighlighting(highlightStyle)];
