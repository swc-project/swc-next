import { ChevronsDownUp, ChevronsUpDown } from "lucide-react";
import { useMemo } from "react";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import {
  astExpandedAtom,
  astExpansionModeAtom,
  astPositionsAtom,
  resultAtom,
  cursorAtom,
  selectionAtom,
  compilingAtom,
} from "../state";
import type { SourceRange } from "../types";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
function rangeOf(value: unknown): SourceRange | undefined {
  return isObject(value) &&
    typeof value.start === "number" &&
    typeof value.end === "number"
    ? { start: value.start, end: value.end }
    : undefined;
}
function literal(value: unknown): string {
  if (typeof value === "bigint") return `${value}n`;
  if (value instanceof RegExp) return value.toString();
  return JSON.stringify(value) ?? String(value);
}

export default function AstTree() {
  const ast = useAtomValue(resultAtom)?.ast;
  const cursor = useAtomValue(cursorAtom).offset;
  const compiling = useAtomValue(compilingAtom);
  const setSelection = useSetAtom(selectionAtom);
  const [expanded, setExpanded] = useAtom(astExpandedAtom);
  const [mode, setMode] = useAtom(astExpansionModeAtom);
  const [positions, setPositions] = useAtom(astPositionsAtom);
  const onSelect = (range: SourceRange) => {
    if (!compiling) setSelection(range);
  };
  const active = useMemo(() => {
    let closest: { path: string; length: number } | undefined;
    function visit(value: unknown, path: string) {
      if (!isObject(value)) return;
      const range = rangeOf(value);
      if (
        range &&
        range.start <= cursor &&
        range.end > cursor &&
        (!closest || range.end - range.start < closest.length)
      )
        closest = { path, length: range.end - range.start };
      for (const [key, child] of Object.entries(value))
        if (isObject(child)) visit(child, `${path}.${key}`);
    }
    visit(ast, "root");
    return closest?.path;
  }, [ast, cursor]);

  function property(
    value: unknown,
    name: string | undefined,
    path: string,
    depth: number,
    parentRange?: SourceRange,
  ) {
    const range = rangeOf(value) ?? parentRange;
    const complex = isObject(value) && !(value instanceof RegExp);
    const entries = complex
      ? Object.entries(value).filter(
          ([key]) =>
            key !== "type" && (positions || (key !== "start" && key !== "end")),
        )
      : [];
    const array = Array.isArray(value);
    const title =
      complex && typeof value.type === "string" ? value.type : undefined;
    const open =
      expanded[path] ??
      (mode === "all" ||
        (mode === "default" &&
          (depth < 2 || Boolean(active?.startsWith(`${path}.`)))));
    const select = () => {
      if (range) onSelect(range);
    };
    if (!complex || entries.length === 0)
      return (
        <div className="w-max min-w-0" key={path}>
          <button
            className="rounded-[3px] bg-transparent px-1 py-0 text-left font-[inherit] whitespace-pre enabled:hover:bg-hover disabled:opacity-100"
            onClick={select}
            disabled={!range}
          >
            {name !== undefined && (
              <>
                <span className="text-ast-key">{name}</span>
                <span className="text-subtle">: </span>
              </>
            )}
            {title && <span className="text-ast-type">{title} </span>}
            <span
              className={
                complex
                  ? "text-subtle"
                  : typeof value === "string"
                    ? "text-ast-string"
                    : typeof value === "number" || typeof value === "bigint"
                      ? "text-ast-number"
                      : typeof value === "boolean"
                        ? "text-ring"
                        : "text-subtle"
              }
            >
              {complex ? (array ? "[]" : "{}") : literal(value)}
            </span>
            <span className="text-subtle">,</span>
          </button>
        </div>
      );
    return (
      <details
        className={`w-max min-w-0 ${path === active ? "[&>summary]:bg-ring/15" : ""}`}
        key={path}
        open={open}
      >
        <summary
          className="relative rounded-[3px] px-1 whitespace-pre hover:bg-hover"
          onClick={(event) => {
            event.preventDefault();
            setExpanded((previous) => ({ ...previous, [path]: !open }));
            select();
          }}
        >
          <span
            aria-hidden="true"
            className={`absolute -left-3 font-semibold ${open ? "text-error" : "text-success"}`}
          >
            {open ? "−" : "+"}
          </span>
          {name !== undefined && (
            <>
              <span className="text-ast-key">{name}</span>
              <span className="text-subtle">: </span>
            </>
          )}
          {title && <span className="text-ast-type">{title} </span>}
          <span className="text-subtle">{array ? "[" : "{"}</span>
          {!open && (
            <>
              <span className="text-subtle">
                {array
                  ? ` ${entries.length} ${entries.length === 1 ? "item" : "items"} `
                  : " … "}
              </span>
              <span className="text-subtle">{array ? "]" : "}"}</span>
            </>
          )}
          {rangeOf(value) && (
            <span className="ml-3 text-xs text-subtle">
              [{range!.start}, {range!.end}]
            </span>
          )}
        </summary>
        {open && (
          <>
            <div className="ml-5.5 border-l border-border pl-1.25">
              {entries.map(([key, child]) =>
                property(child, key, `${path}.${key}`, depth + 1, range),
              )}
            </div>
            <div className="pl-1 text-subtle">{array ? "]" : "}"},</div>
          </>
        )}
      </details>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex h-11 min-h-11 items-center justify-between border-b border-border pr-3.5 pl-5 text-xs text-muted">
        <span>
          ESTree<span className="mx-1.75 text-subtle">·</span>Input AST
        </span>
        <div className="flex items-center gap-0.5">
          <label className="mr-1.5 flex cursor-pointer items-center gap-1.25">
            <input
              className="m-0 size-3.5 accent-ring"
              type="checkbox"
              checked={positions}
              onChange={(event) => setPositions(event.target.checked)}
            />
            Positions
          </label>
          <button
            className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
            title="Expand all"
            aria-label="Expand all"
            onClick={() => {
              setMode("all");
              setExpanded({});
            }}
          >
            <ChevronsUpDown size={14} />
          </button>
          <button
            className="inline-flex size-8 shrink-0 items-center justify-center rounded-[5px] bg-transparent p-0 text-muted enabled:hover:bg-hover enabled:hover:text-heading"
            title="Collapse all"
            aria-label="Collapse all"
            onClick={() => {
              setMode("none");
              setExpanded({});
            }}
          >
            <ChevronsDownUp size={14} />
          </button>
        </div>
      </div>
      <div
        className="min-h-0 flex-1 overflow-auto pt-5.5 pr-6 pb-8 pl-8 font-mono text-sm/[26px]"
        aria-label="Input abstract syntax tree"
      >
        {property(ast, undefined, "root", 0)}
      </div>
    </div>
  );
}
