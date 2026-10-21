import {
  Braces,
  Check,
  ChevronRight,
  CircleAlert,
  Clock3,
  Code2,
  Copy,
  Download,
  ExternalLink,
  FileCode2,
  Moon,
  RotateCcw,
  Sun,
  X,
} from "lucide-react";
import { useEffect, useRef } from "react";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import type { CSSProperties, KeyboardEvent } from "react";
import swcLogo from "../../docs/docs/public/images/swc.png";
import AstTree from "./components/AstTree";
import CodeEditor from "./components/CodeEditor";
import OptionsPanel from "./components/OptionsPanel";
import {
  sourceAtom,
  optionsAtom,
  themeAtom,
  tabAtom,
  resultAtom,
  compilingAtom,
  compilerErrorAtom,
  notificationAtom,
  copiedAtom,
  cursorAtom,
  selectionAtom,
  sourceWidthAtom,
  diagnosticsOpenAtom,
  diagnosticsAtom,
  hasErrorsAtom,
  resetPlaygroundAtom,
} from "./state";
import { useCompiler } from "./useCompiler";

const commit = process.env.SWC_COMMIT;

function stringify(value: unknown) {
  return JSON.stringify(
    value,
    (_, item) =>
      typeof item === "bigint"
        ? `${item}n`
        : item instanceof RegExp
          ? item.toString()
          : item,
    2,
  );
}

export default function App() {
  useCompiler();
  const [source, setSource] = useAtom(sourceAtom);
  const options = useAtomValue(optionsAtom);
  const [theme, setTheme] = useAtom(themeAtom);
  const [tab, setTab] = useAtom(tabAtom);
  const result = useAtomValue(resultAtom);
  const compiling = useAtomValue(compilingAtom);
  const error = useAtomValue(compilerErrorAtom);
  const diagnostics = useAtomValue(diagnosticsAtom);
  const hasErrors = useAtomValue(hasErrorsAtom);
  const [notification, setNotification] = useAtom(notificationAtom);
  const [copied, setCopied] = useAtom(copiedAtom);
  const [cursor, setCursor] = useAtom(cursorAtom);
  const [selection, setSelection] = useAtom(selectionAtom);
  const [sourceWidth, setSourceWidth] = useAtom(sourceWidthAtom);
  const [diagnosticsOpen, setDiagnosticsOpen] = useAtom(diagnosticsOpenAtom);
  const resetPlayground = useSetAtom(resetPlaygroundAtom);
  const editors = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!notification) return;
    const timer = window.setTimeout(() => setNotification(""), 3000);
    return () => window.clearTimeout(timer);
  }, [notification]);
  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), 1800);
    return () => window.clearTimeout(timer);
  }, [copied]);

  const language = options.parser.lang;
  const filename = language === "dts" ? "input.d.ts" : `input.${language}`;
  const editorLanguage = ["ts", "tsx", "dts"].includes(language)
    ? "typescript"
    : "javascript";
  const outputLanguage =
    options.transform.ts && editorLanguage === "typescript"
      ? "javascript"
      : editorLanguage;
  const outputFilename =
    outputLanguage === "typescript"
      ? filename.replace("input.", "output.")
      : language.endsWith("x")
        ? "output.jsx"
        : "output.js";
  const outputAvailable =
    Boolean(result) && !compiling && !error && (tab === "ast" || !hasErrors);
  const status = compiling
    ? result
      ? "Compiling…"
      : "Loading compiler…"
    : error
      ? "Compiler error"
      : hasErrors
        ? `${diagnostics.filter((item) => item.severity === "error").length} errors`
        : "Compiled successfully";

  async function copyOutput() {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(
        tab === "code" ? result.code : stringify(result.ast),
      );
      setCopied(true);
    } catch {
      setNotification(
        "Couldn't copy. Check your browser's clipboard permissions.",
      );
    }
  }
  function downloadOutput() {
    if (!result) return;
    const data = tab === "code" ? result.code : stringify(result.ast);
    const url = URL.createObjectURL(
      new Blob([data], { type: "text/plain;charset=utf-8" }),
    );
    const link = document.createElement("a");
    link.href = url;
    link.download = tab === "ast" ? "ast.json" : outputFilename;
    link.click();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  function selectTab(event: KeyboardEvent<HTMLButtonElement>) {
    if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const next =
        event.key === "Home"
          ? "code"
          : event.key === "End"
            ? "ast"
            : tab === "code"
              ? "ast"
              : "code";
      setTab(next);
      document.getElementById(`tab-${next}`)?.focus();
    }
  }

  return (
    <div className="flex h-dvh min-h-115 flex-col max-[720px]:h-auto max-[720px]:min-h-dvh">
      <header className="flex h-16 shrink-0 items-center justify-between gap-5 border-b border-border px-7 max-[720px]:gap-2 max-[720px]:px-4">
        <img
          className="h-7 w-auto shrink-0"
          src={swcLogo}
          alt="SWC"
          width={4070}
          height={1443}
        />
        <div className="flex items-center gap-1.5 max-[720px]:gap-0">
          {commit && (
            <a
              className="mr-2 rounded px-1 py-1 font-mono text-xs text-muted hover:bg-hover hover:text-heading"
              href={`https://github.com/swc-project/swc-next/commit/${commit}`}
              target="_blank"
              rel="noreferrer"
              aria-label={`SWC commit ${commit}`}
              title={`SWC commit ${commit}`}
            >
              {commit.slice(0, 7)}
            </a>
          )}
          <span className="mr-2.5 flex items-center gap-1.75 text-[13px] text-muted tabular-nums max-[720px]:hidden">
            <Clock3 size={13} />
            {compiling
              ? "Compiling…"
              : result
                ? `${result.duration.toFixed(1)} ms`
                : "— ms"}
          </span>
          <span className="mr-2 h-4.25 w-px bg-border max-[720px]:hidden" />
          <button
            className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
            aria-label="Reset playground"
            title="Reset playground"
            onClick={() => resetPlayground()}
          >
            <RotateCcw size={15} />
          </button>
          <button
            className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
            aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
            title="Toggle theme"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
          >
            {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
          </button>
          <a
            className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted hover:bg-hover hover:text-heading"
            href="https://github.com/swc-project/swc-next"
            target="_blank"
            rel="noreferrer"
            aria-label="SWC Next on GitHub"
            title="GitHub"
          >
            <ExternalLink size={16} />
          </a>
        </div>
      </header>
      <main className="grid min-h-0 flex-1 grid-cols-[276px_minmax(0,1fr)] overflow-hidden bg-card max-[1000px]:grid-cols-[260px_minmax(0,1fr)] max-[720px]:flex max-[720px]:flex-none max-[720px]:flex-col">
        <OptionsPanel />
        <div
          ref={editors}
          className="grid min-h-0 min-w-0 grid-cols-[minmax(0,var(--source-width))_5px_minmax(0,var(--output-width))] max-[720px]:flex max-[720px]:flex-col"
          style={
            {
              "--source-width": `${sourceWidth}fr`,
              "--output-width": `${100 - sourceWidth}fr`,
            } as CSSProperties
          }
        >
          <section
            className="flex min-h-0 min-w-0 flex-col bg-card max-[720px]:h-120 max-[720px]:border-b max-[720px]:border-border"
            aria-label="Input"
            data-panel="input"
          >
            <div className="flex h-13 min-h-13 items-center justify-between gap-2.5 border-b border-border px-5 text-sm/[21px] font-medium text-foreground max-[1000px]:px-4">
              <span className="flex min-w-0 items-center gap-2 whitespace-nowrap [&>svg:first-child]:text-muted">
                <FileCode2 size={14} />
                Input
                <ChevronRight size={11} className="text-subtle" />
                <span className="text-[13px] font-normal text-muted">
                  {filename}
                </span>
              </span>
              <span className="text-xs tracking-[0.5px] text-subtle">
                {language.toUpperCase()}
              </span>
            </div>
            <CodeEditor
              value={source}
              language={editorLanguage}
              theme={theme}
              onChange={setSource}
              onCursor={(offset, line, column) =>
                setCursor({ offset, line, column })
              }
              diagnostics={compiling ? [] : diagnostics}
              selection={selection}
            />
            {!compiling && diagnostics.length > 0 && (
              <div className="border-t border-border">
                <button
                  className="flex h-10 w-full items-center gap-2 bg-card px-5 text-left text-sm [&>svg]:text-error [&>svg:last-child]:ml-auto [&>svg:last-child]:text-muted"
                  aria-expanded={diagnosticsOpen}
                  onClick={() => setDiagnosticsOpen(!diagnosticsOpen)}
                >
                  <CircleAlert size={13} />
                  <span>Diagnostics</span>
                  <span className="rounded bg-hover px-1.25 py-px text-xs text-muted">
                    {diagnostics.length}
                  </span>
                  <ChevronRight
                    size={12}
                    className={diagnosticsOpen ? "rotate-90" : undefined}
                  />
                </button>
                {diagnosticsOpen && (
                  <div className="max-h-40 overflow-auto px-2 pb-2">
                    {diagnostics.map((item, index) => (
                      <button
                        key={index}
                        className="flex w-full items-baseline gap-2.25 rounded bg-transparent px-3 py-1.75 text-left text-sm leading-relaxed hover:bg-hover"
                        onClick={() =>
                          setSelection({ start: item.start, end: item.end })
                        }
                      >
                        <span
                          className={
                            item.severity === "warning"
                              ? "text-xs text-accent"
                              : "text-xs text-error"
                          }
                        >
                          {item.severity}
                        </span>
                        <span>
                          {item.message}
                          {item.help && (
                            <small className="block text-muted">
                              {item.help}
                            </small>
                          )}
                        </span>
                        <span className="ml-auto text-xs text-subtle">
                          {item.start}:{item.end}
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
            <div className="flex h-9 min-h-9 items-center justify-between gap-2.5 overflow-hidden border-t border-border px-5 text-xs/4.5 whitespace-nowrap text-muted tabular-nums max-[1000px]:px-3.5">
              <span>
                Ln {cursor.line}, Col {cursor.column}
              </span>
              <span>
                UTF-8<span className="mx-1.75 text-subtle">·</span>
                {editorLanguage === "typescript" ? "TypeScript" : "JavaScript"}
              </span>
            </div>
          </section>
          <div
            className="z-2 touch-none cursor-col-resize border-x border-border bg-card transition-colors hover:bg-ring active:bg-ring max-[720px]:hidden"
            role="separator"
            aria-label="Resize input and output"
            aria-orientation="vertical"
            aria-valuenow={Math.round(sourceWidth)}
            aria-valuemin={25}
            aria-valuemax={75}
            tabIndex={0}
            onKeyDown={(event) => {
              if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
                event.preventDefault();
                setSourceWidth((width) =>
                  Math.max(
                    25,
                    Math.min(75, width + (event.key === "ArrowLeft" ? -2 : 2)),
                  ),
                );
              }
            }}
            onPointerDown={(event) => {
              event.currentTarget.setPointerCapture(event.pointerId);
              event.preventDefault();
            }}
            onPointerMove={(event) => {
              if (!event.currentTarget.hasPointerCapture(event.pointerId))
                return;
              const bounds = editors.current!.getBoundingClientRect();
              setSourceWidth(
                Math.max(
                  25,
                  Math.min(
                    75,
                    ((event.clientX - bounds.left) / bounds.width) * 100,
                  ),
                ),
              );
            }}
            onPointerUp={(event) =>
              event.currentTarget.releasePointerCapture(event.pointerId)
            }
          />
          <section
            className="flex min-h-0 min-w-0 flex-col bg-card max-[720px]:h-120"
            aria-label="Output"
            data-panel="output"
            aria-busy={compiling}
          >
            <div className="flex h-13 min-h-13 items-center justify-between gap-2.5 border-b border-border pr-2.5 pl-1.5 text-sm/[21px] font-medium text-foreground">
              <div
                className="flex items-stretch gap-0.5 self-stretch"
                role="tablist"
                aria-label="Output format"
              >
                {(["code", "ast"] as const).map((item) => (
                  <button
                    key={item}
                    role="tab"
                    id={`tab-${item}`}
                    aria-controls={`result-${item}`}
                    aria-selected={tab === item}
                    tabIndex={tab === item ? 0 : -1}
                    className="relative flex items-center gap-1.75 bg-transparent px-4 text-sm text-muted hover:text-heading aria-selected:text-heading aria-selected:after:absolute aria-selected:after:right-4 aria-selected:after:-bottom-px aria-selected:after:left-4 aria-selected:after:h-0.5 aria-selected:after:bg-heading"
                    onClick={() => setTab(item)}
                    onKeyDown={selectTab}
                  >
                    {item === "code" ? (
                      <Code2 size={14} />
                    ) : (
                      <Braces size={14} />
                    )}
                    {item === "code" ? "Code" : "AST"}
                  </button>
                ))}
              </div>
              <div className="flex items-center gap-0.5">
                <button
                  className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
                  title="Copy output"
                  aria-label="Copy output"
                  disabled={!outputAvailable}
                  onClick={() => void copyOutput()}
                >
                  {copied ? <Check size={14} /> : <Copy size={14} />}
                </button>
                <button
                  className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
                  title="Download output"
                  aria-label="Download output"
                  disabled={!outputAvailable}
                  onClick={downloadOutput}
                >
                  <Download size={14} />
                </button>
              </div>
            </div>
            <div
              className="flex min-h-0 flex-1 flex-col"
              id={`result-${tab}`}
              role="tabpanel"
              aria-labelledby={`tab-${tab}`}
              tabIndex={0}
            >
              {error ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-sm text-muted [&>svg]:text-error [&>strong]:font-medium [&>strong]:text-foreground [&>p]:max-w-100 [&>p]:text-sm [&>p]:leading-[1.8] [&>p]:whitespace-pre-wrap [&>p]:wrap-anywhere">
                  <CircleAlert size={22} />
                  <strong>Compiler error</strong>
                  <p>{error}</p>
                </div>
              ) : !result ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-sm text-muted [&>svg]:text-subtle [&>strong]:font-medium [&>strong]:text-foreground [&>p]:max-w-100 [&>p]:text-sm [&>p]:leading-[1.8] [&>p]:whitespace-pre-wrap [&>p]:wrap-anywhere">
                  <span className="size-5 animate-spin rounded-full border-2 border-border border-t-accent motion-reduce:animate-none" />
                  <span>Loading WebAssembly compiler…</span>
                </div>
              ) : tab === "ast" ? (
                <AstTree />
              ) : hasErrors ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-sm text-muted [&>svg]:text-subtle [&>strong]:font-medium [&>strong]:text-foreground [&>p]:max-w-100 [&>p]:text-sm [&>p]:leading-[1.8] [&>p]:whitespace-pre-wrap [&>p]:wrap-anywhere">
                  <CircleAlert size={22} />
                  <strong>Check your input</strong>
                  <p>
                    Resolve the diagnostics to generate code.
                    <br />
                    You can still explore the recovered AST.
                  </p>
                </div>
              ) : (
                <CodeEditor
                  value={result.code}
                  language={outputLanguage}
                  theme={theme}
                  readOnly
                />
              )}
            </div>
            <div className="flex h-9 min-h-9 items-center justify-between gap-2.5 overflow-hidden border-t border-border px-5 text-xs/4.5 whitespace-nowrap text-muted tabular-nums max-[1000px]:px-3.5">
              <span
                className={`flex items-center gap-1.75 ${hasErrors || error ? "text-error" : "text-muted"}`}
                role="status"
              >
                <span
                  className={`size-1.25 shrink-0 rounded-full ${compiling ? "animate-pulse bg-accent motion-reduce:animate-none" : hasErrors || error ? "bg-error" : "bg-success"}`}
                />
                {status}
              </span>
              <span className="max-[1000px]:hidden">
                {tab === "ast"
                  ? "Click a node to locate source"
                  : result && !hasErrors
                    ? `${new TextEncoder().encode(result.code).length.toLocaleString()} bytes`
                    : ""}
              </span>
            </div>
          </section>
        </div>
      </main>
      {notification && (
        <div
          className="fixed bottom-7.5 left-1/2 z-10 flex -translate-x-1/2 items-center gap-3 rounded-md border border-border bg-card py-2.25 pr-2.5 pl-4 text-sm shadow-[0_8px_30px_#0003]"
          role="status"
        >
          {notification}
          <button
            className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
            aria-label="Dismiss notification"
            onClick={() => setNotification("")}
          >
            <X size={13} />
          </button>
        </div>
      )}
    </div>
  );
}
