import path from "node:path";
import { defineConfig, mergeRslibConfig } from "@rslib/core";

const root = import.meta.dirname;
const components = {
  core: ["index"],
  parser: ["index"],
  generator: ["index"],
  transformer: ["index"],
  "core-browser": ["index"],
  formatter: ["index", "document"],
  "formatter-prettier-plugin": ["javascript/index", "markdown/index"],
};

const pifeHint = "@swc-next-pife-(start|end)";
const pifeComment = new RegExp(`/\\* ${pifeHint} \\*/`, "g");
const walkerDirectory = path.join(root, "packages", "walker");
const walker = {
  id: "walker",
  format: "esm",
  bundle: false,
  dts: true,
  syntax: "es2022",
  source: {
    tsconfigPath: path.join(walkerDirectory, "tsconfig.json"),
    entry: Object.fromEntries(
      ["index", "child-visitors", "visitor-keys"].map((entry) => [
        entry,
        path.join(walkerDirectory, "src", `${entry}.ts`),
      ]),
    ),
  },
  output: {
    distPath: { root: path.join(walkerDirectory, "dist") },
    minify: false,
  },
};

const baseConfig = defineConfig({
  lib: Object.entries(components).map(([component, entries]) => {
    const directory = path.join(root, "npm", component);
    const source = path.join(directory, "src-js");
    const eagerDecoder = component === "core" || component === "parser";
    return {
      id: component,
      format: "esm",
      bundle: true,
      syntax: "es2022",
      source: {
        entry: Object.fromEntries(
          entries.map((entry) => [entry, path.join(source, `${entry}.js`)]),
        ),
      },
      ...(eagerDecoder && { tools: { rspack: restoreDecoderHints } }),
      output: {
        target: "node",
        distPath: { root: path.join(directory, "dist") },
        // Native/Wasm build tools also write artifacts to dist.
        cleanDistPath: false,
        copy: [{ from: "{**/*.d.ts,binding.js}", context: source }],
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
  }).concat(walker),
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
      id: "formatter-prettier-plugin",
      output: {
        externals: ["../binding.js", "prettier/doc", "prettier/plugins/estree"],
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
