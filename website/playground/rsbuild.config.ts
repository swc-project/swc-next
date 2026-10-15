import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";

export default defineConfig({
  plugins: [pluginReact()],
  source: {
    entry: { index: "./src-js/index.tsx" },
  },
  html: {
    template: "./index.html",
    title: "SWC Next · Playground",
    meta: {
      description:
        "Explore the SWC Next parser, transforms, generated code, and AST in your browser.",
    },
  },
  server: { port: 3000 },
});
