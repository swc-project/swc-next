import path from "node:path";
import { defineConfig } from "@rslib/core";

const root = import.meta.dirname;
const components = ["core", "parser", "generator", "transformer", "core-browser"];

export default defineConfig({
  lib: components.map((component) => {
    const browser = component === "core-browser";
    const directory = path.join(root, "npm", component);
    const source = path.join(directory, "src-js");
    return {
      id: component,
      format: "esm",
      bundle: true,
      syntax: "es2022",
      source: {
        entry: { index: path.join(source, "index.js") },
      },
      output: {
        target: browser ? "web" : "node",
        distPath: { root: path.join(directory, "dist") },
        // Native/Wasm build tools also write artifacts to dist.
        cleanDistPath: false,
        copy: [
          { from: path.join(source, "{*.d.ts,binding.js}"), to: "[name][ext]" },
        ],
        autoExternal: false,
        externals: ["./binding.js", "./swc-next-core.browser.mjs", "@swc-next/walker"],
        minify: true,
      },
    };
  }),
});
