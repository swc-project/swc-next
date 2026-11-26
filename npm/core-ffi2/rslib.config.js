import path from "node:path";
import { defineConfig } from "@rslib/core";

const directory = import.meta.dirname;
const source = path.join(directory, "src-js");
const bundle = { format: "esm", bundle: true, dts: false, syntax: "es2022" };
const output = {
  distPath: { root: path.join(directory, "dist") },
  // The native and Wasm build scripts also write into dist.
  cleanDistPath: false,
  autoExternal: false,
  minify: false,
};

export default defineConfig({
  lib: [
    {
      ...bundle,
      id: "node",
      source: { entry: { index: path.join(source, "index.js") } },
      output: {
        ...output,
        target: "node",
        externals: ["./binding.js"],
        copy: [{ from: path.join(source, "binding.js"), to: "binding.js" }],
      },
    },
    {
      ...bundle,
      id: "browser",
      source: { entry: { browser: path.join(source, "browser.js") } },
      output: {
        ...output,
        target: "web",
        externals: ["./swc-next-core-ffi2.browser.mjs"],
      },
    },
  ],
});
