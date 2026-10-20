import { execFileSync } from "node:child_process";
import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";

function getCommit() {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: import.meta.dirname,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return "";
  }
}

export default defineConfig({
  plugins: [pluginReact()],
  source: {
    entry: { index: "./src-js/index.tsx" },
    define: {
      "process.env.SWC_COMMIT": JSON.stringify(getCommit()),
    },
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
