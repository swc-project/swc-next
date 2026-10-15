import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import type {
  CompileResult,
  Options,
  PlaygroundState,
  SourceRange,
} from "./types";

export const defaultState: PlaygroundState = {
  source: `// Explore the SWC Next compiler.\n// Change an option and see what happens.\n\ninterface Greeting {\n  message: string;\n}\n\nclass Hello implements Greeting {\n  message = "Hello, SWC Next!";\n\n  static {\n    console.log("Ready to compile.");\n  }\n\n  greet(name: string): string {\n    return \`\${this.message} Welcome, \${name}.\`;\n  }\n}\n\nexport const hello = new Hello();\n`,
  options: {
    parser: {
      lang: "ts",
      sourceType: "module",
      preserveParens: true,
      semanticErrors: true,
    },
    transform: { ts: true, es2022: { classStaticBlock: true } },
    codegen: { minify: false, asciiOnly: false, inlineScript: false },
  },
};

export function loadState(): PlaygroundState {
  try {
    const saved = JSON.parse(
      localStorage.getItem("swc-next-playground") ?? "null",
    );
    if (saved && typeof saved.source === "string" && saved.options) {
      const { parser, transform, codegen } = saved.options;
      const validBooleans = [
        parser?.preserveParens,
        parser?.semanticErrors,
        transform?.ts,
        transform?.es2022?.classStaticBlock,
        codegen?.minify,
        codegen?.asciiOnly,
        codegen?.inlineScript,
      ].every((value) => typeof value === "boolean");
      if (
        validBooleans &&
        ["js", "jsx", "ts", "tsx", "dts"].includes(parser.lang) &&
        ["module", "script", "commonjs", "unambiguous"].includes(
          parser.sourceType,
        )
      )
        return saved;
    }
  } catch {
    /* Storage can be unavailable or contain an older state. */
  }
  return defaultState;
}

export const playgroundAtom = atomWithStorage<PlaygroundState>(
  "swc-next-playground",
  defaultState,
  {
    getItem: () => loadState(),
    setItem: (key, value) => {
      try {
        localStorage.setItem(key, JSON.stringify(value));
      } catch {
        /* Storage is optional. */
      }
    },
    removeItem: (key) => {
      try {
        localStorage.removeItem(key);
      } catch {
        /* Storage is optional. */
      }
    },
  },
  { getOnInit: true },
);

export const sourceAtom = atom(
  (get) => get(playgroundAtom).source,
  (get, set, source: string) => {
    set(playgroundAtom, { ...get(playgroundAtom), source });
    set(selectionAtom, undefined);
  },
);
export const optionsAtom = atom(
  (get) => get(playgroundAtom).options,
  (get, set, options: Options) =>
    set(playgroundAtom, { ...get(playgroundAtom), options }),
);

type Theme = "dark" | "light";
const themeValueAtom = atom<Theme>(
  document.documentElement.dataset.theme === "light" ? "light" : "dark",
);
export const themeAtom = atom(
  (get) => get(themeValueAtom),
  (_get, set, theme: Theme) => {
    // Keep the document and stored preference in sync before the next paint.
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("swc-next-theme", theme);
    } catch {
      /* Storage is optional. */
    }
    set(themeValueAtom, theme);
  },
);

export const compilerAtom = atom<{
  result?: CompileResult;
  error?: string;
  compiling: boolean;
}>({ compiling: true });
export const resultAtom = atom((get) => get(compilerAtom).result);
export const compilerErrorAtom = atom((get) => get(compilerAtom).error);
export const compilingAtom = atom((get) => get(compilerAtom).compiling);
export const diagnosticsAtom = atom(
  (get) => get(resultAtom)?.diagnostics ?? [],
);
export const hasErrorsAtom = atom((get) =>
  get(diagnosticsAtom).some((item) => item.severity === "error"),
);

export const tabAtom = atom<"code" | "ast">("code");
export const notificationAtom = atom("");
export const copiedAtom = atom(false);
export const cursorAtom = atom({ offset: 0, line: 1, column: 1 });
export const selectionAtom = atom<SourceRange>();
export const sourceWidthAtom = atom(50);
export const diagnosticsOpenAtom = atom(true);
export const astExpandedAtom = atom<Record<string, boolean>>({});
export const astExpansionModeAtom = atom<"default" | "all" | "none">("default");
export const astPositionsAtom = atom(false);

export const resetPlaygroundAtom = atom(null, (_get, set) => {
  set(playgroundAtom, structuredClone(defaultState));
  set(selectionAtom, undefined);
});
export const resetOptionsAtom = atom(null, (_get, set) => {
  set(optionsAtom, structuredClone(defaultState.options));
});
