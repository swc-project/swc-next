import { once } from "node:events";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";

// This is the encoder re-exported by @swc-next/formatter/document.
import { encodeDoc } from "../../../packages/protocol/document.js";

const require = createRequire(
  new URL(
    "../../../npm/formatter-prettier-plugin/package.json",
    import.meta.url,
  ),
);
const fixture = createRequire(
  new URL("../fixtures/prettier/package.json", import.meta.url),
);
const prettier = require("prettier");
const fixtureVersion = fixture("./package.json").version;
if (prettier.version !== fixtureVersion) {
  throw new Error(
    `Prettier ${prettier.version} differs from the Markdown fixtures (${fixtureVersion})`,
  );
}
const { parsers, printers } = require("prettier/plugins/markdown");
const markdownPrinter = printers[parsers.markdown.astFormat];

const brokenFixture = fileURLToPath(
  new URL(
    "../fixtures/prettier/tests/format/markdown/broken-plugins/missing-comments.md",
    import.meta.url,
  ),
);

function observeMarkdown(fences, errors) {
  const observer = {
    parsers: { markdown: parsers.markdown },
    printers: {
      [parsers.markdown.astFormat]: {
        ...markdownPrinter,
        preprocess(ast, settings) {
          // Capture the installed printer after Prettier adds its front matter
          // handler. The proxy forwards the original embed traversal hooks.
          const printer = settings.printer;
          settings.printer = {
            ...printer,
            embed: new Proxy(printer.embed, {
              apply(target, receiver, args) {
                return observe(Reflect.apply(target, receiver, args), ...args);
              },
            }),
          };
          return markdownPrinter.preprocess(ast, settings);
        },
      },
    },
  };
  return observer;

  function observe(render, path, settings) {
    const node = path.node;
    const frontMatter = node.type === "frontMatter";
    if (!frontMatter && (node.type !== "code" || node.isIndented || !node.lang))
      return render;
    const record = {
      language: frontMatter ? node.language : node.lang,
      // The Markdown AST's value omits the newline before the closing delimiter.
      source: frontMatter
        ? node.raw.slice(
            node.raw.indexOf("\n") + 1,
            node.raw.lastIndexOf("\n") + 1,
          )
        : node.value,
      doc: null,
    };
    fences.push(record);
    if (!render) return;

    return async (textToDoc, ...args) => {
      let guest;
      let children;
      const fence = await render(
        async (text, childOptions) => {
          const plugins = settings.plugins.filter(
            (plugin) => plugin !== observer,
          );
          if (childOptions.parser === "markdown") {
            children = [];
            plugins.push(observeMarkdown(children, errors));
          }
          guest = await textToDoc(frontMatter ? record.source : text, {
            ...childOptions,
            plugins,
          });
          return guest;
        },
        ...args,
      );
      if (guest !== undefined && fence && children) {
        record.children = children;
      } else if (
        fence &&
        (guest !== undefined ||
          (frontMatter && /^[ \t\n]*$/u.test(record.source)))
      ) {
        try {
          record.doc = Buffer.from(encodeDoc(guest ?? "").buffer).toString(
            "base64",
          );
        } catch (error) {
          // Prettier catches errors from embed as source fallbacks.
          errors.push(
            new Error(`Cannot encode the ${record.language} document`, {
              cause: error,
            }),
          );
        }
      }
      return fence;
    };
  }
}

async function collectFences(source, options, filepath) {
  const roots = [];
  const errors = [];
  const plugins = [...(options.plugins ?? [])];
  if (filepath && resolve(filepath) === brokenFixture) {
    plugins.push(
      fixture(
        "./tests/config/prettier-plugins/prettier-plugin-missing-comments/index.cjs",
      ),
    );
  }
  // Keep the public parser name and leave the guest registry and ordering intact.
  await prettier.format(source, {
    ...options,
    filepath,
    parser: "markdown",
    plugins: [...plugins, observeMarkdown(roots, errors)],
  });
  if (errors.length) throw errors[0];

  // The native dispatcher uses each Markdown record's 1-based ID to find its children.
  const fences = [];
  function append(records, parent) {
    for (const { children, ...record } of records) {
      const id = fences.push({
        ...record,
        parent,
        markdown: Boolean(children),
      });
      if (children) append(children, id);
    }
  }
  append(roots, 0);
  return { fences };
}

const isObject = (value) =>
  value !== null && typeof value === "object" && !Array.isArray(value);
const input = createInterface({ input: process.stdin, crlfDelay: Infinity });
let lineNumber = 0;
for await (const line of input) {
  lineNumber++;
  try {
    const request = JSON.parse(line);
    if (!isObject(request) || typeof request.source !== "string") {
      throw new TypeError("Expected a Markdown source string");
    }
    const { source, options = {} } = request;
    if (!isObject(options)) throw new TypeError("Expected formatting options");
    const filepath = request.filepath ?? options.filepath ?? undefined;
    if (filepath !== undefined && typeof filepath !== "string") {
      throw new TypeError("Expected a filepath string");
    }
    if (options.plugins !== undefined && !Array.isArray(options.plugins)) {
      throw new TypeError("Expected the Prettier plugins list to be an array");
    }
    const result = await collectFences(source, options, filepath);
    if (!process.stdout.write(`${JSON.stringify(result)}\n`)) {
      await once(process.stdout, "drain");
    }
  } catch (error) {
    throw new Error(`Prettier Markdown fence request ${lineNumber} failed`, {
      cause: error,
    });
  }
}
