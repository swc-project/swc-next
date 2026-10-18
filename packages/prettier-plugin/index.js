import { decode } from "./decode.js";
import { encodeOptions } from "./options.js";
export { options } from "./options.js";

let binding;
const loadBinding = () => (binding ??= import("@swc-next/formatter"));

// Values from the shared parser option wire protocol.
function sourceTypeForPath(path) {
  if (/\.(?:mjs|mts)$/iu.test(path)) return 1; // Module
  if (/\.(?:cjs|cts)$/iu.test(path)) return 3; // CommonJs
  return 4; // Unambiguous
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
      const formatOptions = encodeOptions(options);
      const { formatDocBufferSync } = await loadBinding();
      const path = options.filepath ?? "";
      const declaration = lang === "ts" && /\.d\.(?:ts|mts|cts)$/iu.test(path);
      const language = declaration ? 5 : { jsx: 3, ts: 2, tsx: 4 }[lang];
      const sourceType = sourceTypeForPath(path);
      // Existing parser option wire: present + source type + language.
      const parserOptions = 1 | (sourceType << 1) | (language << 4);
      const buffer = formatDocBufferSync(source, parserOptions, formatOptions);
      return {
        type: "SWCFMT",
        start: 0,
        end: source.length,
        doc: decode(buffer, source, options),
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
