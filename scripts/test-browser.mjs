#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const packageJson = fileURLToPath(import.meta.resolve("@rstest/core/package.json"));
const cli = resolve(dirname(packageJson), "bin/rstest.js");
const forwarded = process.argv.slice(2);

for (const project of ["default", "browser"]) {
  const result = spawnSync(
    process.execPath,
    [
      cli,
      "run",
      "--config",
      "rstest.browser.config.js",
      "--project",
      project,
      ...forwarded,
    ],
    { stdio: "inherit" },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
