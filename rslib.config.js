import path from "node:path";
import { defineConfig, mergeRslibConfig } from "@rslib/core";

const root = import.meta.dirname;
const components = [
  "core",
  "parser",
  "generator",
  "transformer",
  "core-browser",
  "formatter",
  "formatter-js",
  "formatter-markdown",
  "formatter-prettier-plugin",
];

const pifeHint = "@swc-next-pife-(start|end)";
const pifeComment = new RegExp(`/\\* ${pifeHint} \\*/`, "g");

const baseConfig = defineConfig({
  lib: components.map((component) => {
    const directory = path.join(root, "npm", component);
    const source = path.join(directory, "src-js");
    const eagerDecoder = component === "core" || component === "parser";
    return {
      id: component,
      format: "esm",
      bundle: true,
      syntax: "es2022",
      source: {
        entry: {
          index: path.join(source, "index.js"),
        },
      },
      ...(eagerDecoder && { tools: { rspack: restoreDecoderHints } }),
      output: {
        target: "node",
        distPath: { root: path.join(directory, "dist") },
        // Native/Wasm build tools also write artifacts to dist.
        cleanDistPath: false,
        copy: [
          { from: path.join(source, "{*.d.ts,binding.js}"), to: "[name][ext]" },
        ],
        autoExternal: false,
        externals: [
          "./binding.js",
          "./swc-next-core.browser.mjs",
          "@swc-next/walker",
        ],
        minify: {
          jsOptions: {
            minimizerOptions: {
              compress: {
                module: true,
                toplevel: false,
                inline: 0,
                reduce_vars: false,
              },
              mangle: { toplevel: false },
              ...(eagerDecoder && {
                format: { comments: { regex: pifeHint } },
              }),
            },
          },
        },
      },
    };
  }),
});

export default mergeRslibConfig(baseConfig, {
  lib: [
    {
      id: "core-browser",
      output: {
        target: "web",
      },
    },
    {
      id: "formatter",
      source: {
        entry: {
          document: path.join(root, "npm/formatter/src-js/document.js"),
        },
      },
    },
    {
      id: "formatter-prettier-plugin",
      output: {
        externals: [
          "@swc-next/formatter-js",
          "prettier/doc",
          "prettier/plugins/estree",
        ],
      },
    },
  ],
});

// =================================== AI Slop ================================================

function restoreDecoderHints(_, { appendPlugins, rspack }) {
  const name = "DecoderCompilationHints";
  appendPlugins({
    apply(compiler) {
      compiler.hooks.thisCompilation.tap(name, (compilation) => {
        // Restore parentheses after SWC minifies the bundle and before source maps are emitted.
        compilation.hooks.processAssets.tap(
          {
            name,
            stage: rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE + 1,
          },
          () => {
            const { source } = compilation.getAsset("index.js");
            const code = source.source().toString();
            const hints = [...code.matchAll(pifeComment)];
            if (hints.length !== 10) {
              throw new Error(
                "Expected five pairs of decoder compilation hints",
              );
            }
            const updated = new rspack.sources.ReplaceSource(source);
            let depth = 0;
            for (const hint of hints) {
              const [comment, edge] = hint;
              const start = hint.index;
              const end = start + comment.length;
              const opening = edge === "start";
              const nextToFunction = opening
                ? /^\s*function\b/.test(code.slice(end))
                : /}\s*$/.test(code.slice(0, start));
              depth += opening ? 1 : -1;
              if (depth < 0 || !nextToFunction) {
                throw new Error("Misplaced decoder compilation hint");
              }
              updated.replace(start, end - 1, opening ? "(" : ")");
            }
            if (depth !== 0) {
              throw new Error("Unbalanced decoder compilation hints");
            }
            compilation.updateAsset("index.js", updated);
          },
        );
      });
    },
  });
}
