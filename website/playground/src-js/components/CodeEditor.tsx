import { useEffect, useRef } from "react";
import * as monaco from "monaco-editor/editor.js";
import "monaco-editor/features/register.all.js";
import "monaco-editor/languages/definitions/typescript/register.js";
import "monaco-editor/languages/definitions/javascript/register.js";
import type { Diagnostic } from "@swc-next/decoder";
import type { SourceRange } from "../types";

globalThis.MonacoEnvironment = {
  getWorker: () =>
    new Worker(
      new URL("monaco-editor/editor/editor.worker.js", import.meta.url),
      { type: "module" },
    ),
};

monaco.editor.defineTheme("workspace-dark", {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "comment", foreground: "8a8f98" },
    { token: "keyword", foreground: "bab6e3" },
    { token: "string", foreground: "b5c8b4" },
    { token: "number", foreground: "d4ba8c" },
    { token: "type.identifier", foreground: "d0c8ad" },
  ],
  colors: {
    "editor.background": "#0b0c0d",
    "editor.foreground": "#d0d6e0",
    "editorLineNumber.foreground": "#747983",
    "editorLineNumber.activeForeground": "#a2a6af",
    "editor.lineHighlightBackground": "#111214",
    "editor.selectionBackground": "#35364b",
    "editor.inactiveSelectionBackground": "#272936",
    "editorWidget.background": "#191a1b",
    "editorWidget.border": "#292c31",
  },
});
monaco.editor.defineTheme("workspace-light", {
  base: "vs",
  inherit: true,
  rules: [],
  colors: {
    "editor.background": "#ffffff",
    "editor.lineHighlightBackground": "#f7f7f8",
    "editorLineNumber.foreground": "#797d85",
  },
});

interface Props {
  value: string;
  language: string;
  theme: "dark" | "light";
  readOnly?: boolean;
  onChange?: (value: string) => void;
  onCursor?: (offset: number, line: number, column: number) => void;
  diagnostics?: Diagnostic[];
  selection?: SourceRange;
}

export default function CodeEditor({
  value,
  language,
  theme,
  readOnly = false,
  onChange,
  onCursor,
  diagnostics = [],
  selection,
}: Props) {
  const container = useRef<HTMLDivElement>(null);
  const instance = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const callbacks = useRef({ onChange, onCursor });
  callbacks.current = { onChange, onCursor };

  useEffect(() => {
    const model = monaco.editor.createModel(value, language);
    const editor = monaco.editor.create(container.current!, {
      model,
      readOnly,
      theme: `workspace-${theme}`,
      automaticLayout: true,
      fontSize: 15,
      lineHeight: 26,
      fontFamily: '"SFMono-Regular", Consolas, "Liberation Mono", monospace',
      padding: { top: 24, bottom: 24 },
      guides: { bracketPairs: false },
      bracketPairColorization: { enabled: false },
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      smoothScrolling: true,
      renderLineHighlight: readOnly ? "none" : "line",
      lineNumbersMinChars: 3,
      folding: true,
      glyphMargin: false,
      overviewRulerLanes: 0,
      hideCursorInOverviewRuler: true,
      contextmenu: true,
      wordWrap: "off",
      tabSize: 2,
      ariaLabel: readOnly ? "Generated code" : "Source code",
    });
    instance.current = editor;
    const content = editor.onDidChangeModelContent(() =>
      callbacks.current.onChange?.(model.getValue()),
    );
    const cursor = editor.onDidChangeCursorPosition(({ position }) =>
      callbacks.current.onCursor?.(
        model.getOffsetAt(position),
        position.lineNumber,
        position.column,
      ),
    );
    return () => {
      content.dispose();
      cursor.dispose();
      editor.dispose();
      model.dispose();
      instance.current = null;
    };
    // The model lives for this panel's lifetime; prop changes are applied below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const editor = instance.current;
    if (editor && editor.getValue() !== value) editor.setValue(value);
  }, [value]);
  useEffect(() => {
    const model = instance.current?.getModel();
    if (model) monaco.editor.setModelLanguage(model, language);
  }, [language]);
  useEffect(() => {
    monaco.editor.setTheme(`workspace-${theme}`);
  }, [theme]);
  useEffect(() => {
    const model = instance.current?.getModel();
    if (!model) return;
    monaco.editor.setModelMarkers(
      model,
      "swc-next",
      diagnostics.map((item) => {
        const start = model.getPositionAt(item.start);
        const end = model.getPositionAt(Math.max(item.end, item.start + 1));
        return {
          startLineNumber: start.lineNumber,
          startColumn: start.column,
          endLineNumber: end.lineNumber,
          endColumn: end.column,
          message: item.message + (item.help ? `\n${item.help}` : ""),
          severity:
            item.severity === "error"
              ? monaco.MarkerSeverity.Error
              : item.severity === "warning"
                ? monaco.MarkerSeverity.Warning
                : monaco.MarkerSeverity.Info,
        };
      }),
    );
  }, [diagnostics, value]);
  useEffect(() => {
    const editor = instance.current;
    const model = editor?.getModel();
    if (!selection || !editor || !model) return;
    const start = model.getPositionAt(selection.start);
    const end = model.getPositionAt(selection.end);
    const range = new monaco.Range(
      start.lineNumber,
      start.column,
      end.lineNumber,
      end.column,
    );
    editor.setSelection(range);
    editor.revealRangeInCenterIfOutsideViewport(range);
  }, [selection]);

  return (
    <div
      ref={container}
      className="h-full min-h-0 min-w-0 flex-1 overflow-hidden"
    />
  );
}
