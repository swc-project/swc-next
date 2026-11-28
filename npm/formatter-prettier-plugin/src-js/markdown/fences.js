import { builders, printer, utils } from "prettier/doc";
import { resolveEmbeddedLanguage } from "./embedded-language.js";

const noRun = { prefix: 0, suffix: 0, longest: 0, whole: 0, deferred: 0 };

export function embedBlock(block, options) {
  const depth = options.__swcMarkdownEmbedDepth ?? 0;
  const frontMatter = "opening" in block;
  if (frontMatter && !block.body) return printFrontMatter(block, "");
  const child =
    frontMatter && block.language === "yaml"
      ? { parser: "yaml" }
      : resolveEmbeddedLanguage(block.language, options.plugins);
  if (!child) return;

  return async (textToDoc) => {
    try {
      const doc = await textToDoc(block.body, {
        ...child,
        __swcMarkdownEmbedDepth: depth + 1,
      });
      if (frontMatter) {
        // A guest can return either a fragment or a complete document with a
        // significant final newline, as in a YAML keep-chomped block scalar.
        const { formatted } = printer.printDocToString(doc, {
          ...options,
          endOfLine: "lf",
        });
        return printFrontMatter(block, formatted);
      }
      const guest = utils.replaceEndOfLine(doc);
      const info =
        block.sourceInfo ??
        (block.meta ? `${block.language} ${block.meta}` : block.language);
      const marker = info.includes("`") ? "~" : "`";
      const { longest, deferred } = describeFenceRuns(guest, marker);
      const delimiter = marker.repeat(Math.max(3, longest + deferred + 1));
      return builders.markAsRoot([
        delimiter,
        info.startsWith(marker) ? " " : "",
        info,
        builders.hardline,
        guest,
        builders.hardline,
        delimiter,
      ]);
    } catch {
      // The parent label already contains the logical source fallback.
      return undefined;
    }
  };
}

function printFrontMatter(block, body) {
  return builders.markAsRoot([
    block.opening,
    builders.hardline,
    utils.replaceEndOfLine(body),
    body && !body.endsWith("\n") ? builders.literalline : "",
    block.closing,
  ]);
}

// A delimiter must be safe in every layout the enclosing document might choose.
// Summarize consecutive marker runs through both branches and adjacent strings,
// including split strings and layouts that emit nothing between them.
function describeFenceRuns(doc, marker) {
  const runs = new Map();
  const sequence = (parts) =>
    parts.reduce((left, part) => concatRuns(left, runs.get(part)), noRun);
  utils.traverseDoc(
    doc,
    (part) => !runs.has(part),
    (part) => {
      if (runs.has(part)) return;
      let run;
      if (typeof part === "string") {
        // A later trim may remove horizontal whitespace at a Doc boundary.
        const text = part.replace(/[ \t]+$/u, "");
        let prefix = 0;
        while (text[prefix] === marker) prefix++;
        let suffix = prefix;
        let longest = prefix;
        for (let index = prefix; index < text.length; index++) {
          suffix = text[index] === marker ? suffix + 1 : 0;
          longest = Math.max(longest, suffix);
        }
        run = {
          ...noRun,
          prefix,
          suffix,
          longest,
          whole: prefix === text.length ? prefix : -Infinity,
        };
      } else if (Array.isArray(part) || part.type === "fill") {
        run = sequence(Array.isArray(part) ? part : part.parts);
      } else if (part.type === "if-break") {
        run = alternateRuns(
          runs.get(part.breakContents),
          runs.get(part.flatContents),
        );
      } else if (part.type === "group" && part.expandedStates?.length) {
        run = part.expandedStates
          .map((state) => runs.get(state))
          .reduce(alternateRuns);
      } else if (part.type === "line") {
        run = part.hard ? { ...noRun, whole: -Infinity } : noRun;
      } else if (part.type === "line-suffix") {
        const suffix = runs.get(part.contents);
        run = { ...noRun, deferred: suffix.longest + suffix.deferred };
      } else {
        run = runs.get(part.contents) ?? noRun;
      }
      runs.set(part, run);
    },
    true,
  );
  return runs.get(doc);
}

function concatRuns(left, right) {
  return {
    prefix: Math.max(left.prefix, left.whole + right.prefix),
    suffix: Math.max(right.suffix, right.whole + left.suffix),
    longest: Math.max(left.longest, right.longest, left.suffix + right.prefix),
    whole: left.whole + right.whole,
    deferred: left.deferred + right.deferred,
  };
}

function alternateRuns(first, second) {
  return {
    prefix: Math.max(first.prefix, second.prefix),
    suffix: Math.max(first.suffix, second.suffix),
    longest: Math.max(first.longest, second.longest),
    whole: Math.max(first.whole, second.whole),
    deferred: Math.max(first.deferred, second.deferred),
  };
}
