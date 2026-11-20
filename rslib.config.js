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
  "formatter-prettier-plugin",
];

const baseConfig = defineConfig({
  lib: components.map((component) => {
    const directory = path.join(root, "npm", component);
    const source = path.join(directory, "src-js");
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
          "@swc-next/ast",
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
        externals: ["@swc-next/formatter-js", "prettier/doc", "prettier/plugins/estree"],
      },
    },
  ],
});
