import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags } from "@lezer/highlight";

const editorTheme = EditorView.theme(
  {
    "&": {
      color: "#b8c2d1",
      backgroundColor: "#0d141f",
      height: "100%",
      fontSize: "13px",
    },
    ".cm-scroller": {
      fontFamily:
        "'Cascadia Code', 'JetBrains Mono', 'SFMono-Regular', Consolas, monospace",
      lineHeight: "1.62",
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
        backgroundColor: "rgba(45, 124, 255, 0.4)",
      },
    ".cm-gutters": {
      backgroundColor: "transparent",
      color: "#566177",
      border: "none",
      paddingLeft: "8px",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(255, 255, 255, 0.04)",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "rgba(255, 255, 255, 0.04)",
      color: "#aab6ca",
    },
    ".cm-lineNumbers .cm-gutterElement": {
      padding: "0 10px 0 6px",
      minWidth: "34px",
    },
    ".cm-matchingBracket": {
      backgroundColor: "rgba(45, 124, 255, 0.22)",
      outline: "1px solid rgba(90, 162, 255, 0.6)",
    },
    ".cm-tooltip": {
      backgroundColor: "#17202e",
      border: "1px solid rgba(255, 255, 255, 0.13)",
      color: "#dbe2ee",
      borderRadius: "8px",
      boxShadow: "0 12px 30px rgba(0, 0, 0, 0.5)",
    },
    ".cm-tooltip-autocomplete ul li[aria-selected]": {
      backgroundColor: "rgba(45, 124, 255, 0.35)",
      color: "#ffffff",
    },
    ".cm-tooltip-autocomplete ul li": {
      padding: "3px 8px",
    },
    ".cm-panels": {
      backgroundColor: "#17202e",
      color: "#dbe2ee",
    },
    ".cm-searchMatch": {
      backgroundColor: "rgba(61, 220, 132, 0.22)",
      outline: "1px solid rgba(61, 220, 132, 0.5)",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "rgba(45, 124, 255, 0.32)",
    },
    ".cm-lintRange-error": {
      backgroundImage:
        "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='6' height='3'%3E%3Cpath d='M0 3 L2 0 L4 3 L6 0' fill='none' stroke='%23e5484d' stroke-width='1'/%3E%3C/svg%3E\")",
      backgroundRepeat: "repeat-x",
      backgroundPosition: "bottom",
    },
    ".cm-tooltip-lint": {
      backgroundColor: "#17202e",
    },
  },
  { dark: true },
);

const highlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: "#4cc9f0", fontWeight: "600" },
  { tag: tags.string, color: "#c4e65a" },
  { tag: tags.number, color: "#ffd166" },
  { tag: tags.variableName, color: "#8fe388" },
  { tag: tags.operator, color: "#b8c2d1" },
  { tag: tags.punctuation, color: "#94a3b8" },
  { tag: tags.comment, color: "#5f6b80", fontStyle: "italic" },
  { tag: tags.invalid, color: "#e5484d" },
]);

export const darculaTheme = [editorTheme, syntaxHighlighting(highlightStyle)];
