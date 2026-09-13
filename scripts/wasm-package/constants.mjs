import { fileURLToPath } from "node:url";

export const TARGET = "wasm32-unknown-unknown";
export const REPOSITORY_ROOT = fileURLToPath(new URL("../../", import.meta.url));

export const DEFAULT_OPTIONS = {
  packageJsonPath: "package.json",
  npmDir: "npm",
  outputDir: "./artifacts",
  dryRun: false,
  ghRelease: true,
  skipOptionalPublish: false,
  tagStyle: "lerna",
};
