import { useEffect, useRef } from "react";

import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, indentOnInput } from "@codemirror/language";
import { lintKeymap } from "@codemirror/lint";
import { EditorState } from "@codemirror/state";
import {
  crosshairCursor,
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  placeholder,
  rectangularSelection,
} from "@codemirror/view";

import type { TableInfo } from "../../lib/types";
import { fakeSqlCompletions } from "../../editor/completion";
import { darculaTheme } from "../../editor/darculaTheme";
import { fakeSqlLanguage } from "../../editor/fakesqlLanguage";
import { fakeSqlLinter } from "../../editor/linter";

interface QueryEditorProps {
  query: string;
  tables: TableInfo[];
  /** While a modal dialog is open the editor must not intercept Ctrl+Enter. */
  inactive?: boolean;
  onChange: (query: string) => void;
  onRun: () => void;
  onCursor?: (position: { line: number; column: number }) => void;
}

export function QueryEditor({
  query,
  tables,
  inactive = false,
  onChange,
  onRun,
  onCursor,
}: QueryEditorProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const tablesRef = useRef(tables);
  const onChangeRef = useRef(onChange);
  const onRunRef = useRef(onRun);
  const onCursorRef = useRef(onCursor);
  const inactiveRef = useRef(inactive);

  tablesRef.current = tables;
  onChangeRef.current = onChange;
  onRunRef.current = onRun;
  onCursorRef.current = onCursor;
  inactiveRef.current = inactive;

  useEffect(() => {
    if (!hostRef.current) return;

    const state = EditorState.create({
      doc: query,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        bracketMatching(),
        closeBrackets(),
        rectangularSelection(),
        crosshairCursor(),
        highlightActiveLine(),
        autocompletion({
          override: [fakeSqlCompletions(() => tablesRef.current)],
          activateOnTyping: true,
        }),
        fakeSqlLanguage,
        darculaTheme,
        fakeSqlLinter,
        placeholder("Écrivez une requête FakeSQL… ex. afficher tous les 'users'"),
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...completionKeymap,
          ...lintKeymap,
          indentWithTab,
        ]),
        EditorView.domEventHandlers({
          keydown(event) {
            if (inactiveRef.current) return false;
            if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
              event.preventDefault();
              event.stopPropagation();
              onRunRef.current();
              return true;
            }
            return false;
          },
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChangeRef.current(update.state.doc.toString());
          }
          if (onCursorRef.current && (update.docChanged || update.selectionSet)) {
            const head = update.state.selection.main.head;
            const line = update.state.doc.lineAt(head);
            onCursorRef.current({ line: line.number, column: head - line.from + 1 });
          }
        }),
      ],
    });

    const view = new EditorView({ state, parent: hostRef.current });
    viewRef.current = view;
    view.focus();

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // The editor is re-created for each tab via the component key.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current !== query) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: query },
      });
    }
  }, [query]);

  return <div className="editor-host" ref={hostRef} />;
}
