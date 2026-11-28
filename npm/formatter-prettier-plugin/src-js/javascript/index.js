import { decodeDoc } from "../decode.js";
import { formatJsDoc } from "./format.js";
import { toFormatterConfig } from "./options.js";
export { options } from "./options.js";

function sourceTypeForPath(path) {
  if (/\.(?:mjs|mts)$/iu.test(path)) return "module";
  if (/\.(?:cjs|cts)$/iu.test(path)) return "commonjs";
  return "unambiguous";
}

function parser(lang) {
  return {
    astFormat: "SWCFMT",
    locStart: (root) => root.start,
    locEnd: (root) => root.end,
    async parse(source, options) {
      if (
        (options.rangeStart ?? 0) > 0 ||
        (options.rangeEnd ?? Infinity) < source.length
      ) {
        throw new Error("SWCFMT currently supports whole-file formatting only");
      }
      if (
        options.requirePragma ||
        options.insertPragma ||
        options.checkIgnorePragma
      ) {
        throw new Error("SWCFMT does not yet support pragma options");
      }
      const formatOptions = toFormatterConfig(options);
      const { formatJsDocBufferSync } = await import("../binding.js");
      const path = options.filepath ?? "";
      const declaration = lang === "ts" && /\.d\.(?:ts|mts|cts)$/iu.test(path);
      const language = declaration ? "dts" : lang;
      const sourceType = sourceTypeForPath(path);
      const parserOptions = { lang: language, sourceType };
      let buffer;
      try {
        buffer = formatJsDoc(
          formatJsDocBufferSync,
          source,
          parserOptions,
          formatOptions,
        );
      } catch (error) {
        if (
          error instanceof SyntaxError &&
          Number.isInteger(error.start) &&
          Number.isInteger(error.end)
        ) {
          const location = (offset) => {
            const lines = source
              .slice(0, offset)
              .split(/\r\n|[\n\r\u2028\u2029]/u);
            return { line: lines.length, column: lines.at(-1).length + 1 };
          };
          error.loc = {
            start: location(error.start),
            end: location(error.end),
          };
        }
        throw error;
      }
      return {
        type: "SWCFMT",
        start: 0,
        end: source.length,
        doc: decodeDoc(buffer, options).doc,
      };
    },
  };
}

export const parsers = {
  swc: parser("jsx"),
  "swc-ts": parser("ts"),
  "swc-tsx": parser("tsx"),
};

export const printers = {
  SWCFMT: {
    print: ({ node }) => node.doc,
    getVisitorKeys: () => [],
  },
};

export const languages = [
  {
    name: "JavaScript",
    parsers: ["swc"],
    extensions: [".js", ".jsx", ".mjs", ".cjs"],
    vscodeLanguageIds: ["javascript", "javascriptreact"],
  },
  {
    name: "TypeScript",
    parsers: ["swc-ts"],
    extensions: [".ts", ".mts", ".cts"],
    vscodeLanguageIds: ["typescript"],
  },
  {
    name: "TSX",
    parsers: ["swc-tsx"],
    extensions: [".tsx"],
    vscodeLanguageIds: ["typescriptreact"],
  },
];
