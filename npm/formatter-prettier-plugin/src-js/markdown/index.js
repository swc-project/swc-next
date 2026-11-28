import { utils } from "prettier/doc";
import { decodeDoc } from "../decode.js";
import { decodeEmbeddedBlocks, formatMarkdownDocBuffer } from "./format.js";
import { embedBlock } from "./fences.js";

const astFormat = "SWCFMT_MARKDOWN";
const maxEmbedDepth = 8;

export const languages = [
  {
    name: "Markdown",
    parsers: ["swc-markdown"],
    extensions: [
      ".md",
      ".markdown",
      ".mdown",
      ".mdwn",
      ".mkd",
      ".mkdn",
      ".mkdown",
      ".ronn",
    ],
    vscodeLanguageIds: ["markdown"],
  },
];

export const parsers = {
  "swc-markdown": {
    astFormat,
    locStart: (node) => node.start,
    locEnd: (node) => node.end,
    async parse(source, options) {
      if (
        (options.rangeStart ?? 0) > 0 ||
        (options.rangeEnd ?? Infinity) < source.length
      ) {
        throw new Error("SWC Markdown supports whole-file formatting only");
      }
      if (
        options.requirePragma ||
        options.insertPragma ||
        options.checkIgnorePragma
      ) {
        throw new Error("SWC Markdown does not yet support pragma options");
      }
      const { formatMarkdownDocBufferSync } = await import("../binding.js");
      const buffer = formatMarkdownDocBuffer(
        formatMarkdownDocBufferSync,
        source,
        {
          proseWrap: options.proseWrap,
          singleQuote: options.singleQuote,
          lineWidth: options.printWidth,
          indentWidth: options.tabWidth,
          indentStyle: options.useTabs ? "tab" : "space",
          lineEnding: options.endOfLine,
          includeEmbeds:
            options.embeddedLanguageFormatting !== "off" &&
            (options.__swcMarkdownEmbedDepth ?? 0) < maxEmbedDepth,
        },
      );
      const { doc, labelData } = decodeDoc(buffer, options);
      return {
        type: astFormat,
        start: 0,
        end: source.length,
        doc,
        embeds: decodeEmbeddedBlocks(labelData, source),
      };
    },
  },
};

export const printers = {
  [astFormat]: {
    getVisitorKeys: (node) => (node.type === astFormat ? ["embeds"] : []),
    embed: (path, options) => {
      if (path.node.type !== astFormat) return embedBlock(path.node, options);
    },
    print(path, _options, print) {
      const { node } = path;
      // Prettier returns its saved embedded Doc instead of calling this branch
      // for a successful embed. Empty guest output still has block delimiters.
      if (node.type !== astFormat) return "";
      if (!node.embeds.length) return node.doc;
      const embedded = path.map(print, "embeds");
      return utils.mapDoc(node.doc, (doc) =>
        doc.type === "label" ? embedded[doc.label] || doc.contents : doc,
      );
    },
  },
};
