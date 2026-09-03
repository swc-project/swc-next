#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { wasmApiPackages } from "./version.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../..");

for (const { directory } of wasmApiPackages()) {
  const manifestPath = path.join(repositoryRoot, directory, "package.json");
  const packageJson = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const binding = packageJson.wasmBinding;
  if (typeof binding !== "string" || binding.length === 0) {
    throw new Error(`${packageJson.name} must declare wasmBinding`);
  }
  packageJson.dependencies = {
    ...(packageJson.dependencies ?? {}),
    [binding]: packageJson.version,
  };
  fs.writeFileSync(manifestPath, `${JSON.stringify(packageJson, null, 2)}\n`);
}
