#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { collectArtifacts, createNpmDirs } from "./wasm-package/index.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "..");
const components = ["codegen", "core", "parser"];

for (const component of components) {
  await linkBinding(component);
}

run(
  "pnpm",
  components.flatMap((component) => ["--filter", `@swc-next/${component}`]).concat("test"),
  { ...process.env, NAPI_RS_FORCE_WASI: "error" },
);
run(process.execPath, ["--test", "npm/core-wasm/test/core.test.js"], process.env);

async function linkBinding(component) {
  const packageJsonPath = path.join(
    repositoryRoot,
    "npm",
    component,
    "package.json",
  );
  const outputDir = path.join(repositoryRoot, "target", "wasm-package", component);
  const npmDir = path.join(repositoryRoot, "target", "wasm-test", component);
  const { bindingDirectory } = await createNpmDirs({
    cwd: repositoryRoot,
    npmDir,
    packageJsonPath,
  });
  await collectArtifacts({
    cwd: repositoryRoot,
    npmDir,
    outputDir,
    packageJsonPath,
  });

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
  const bindingName = packageJson.wasm.packageName;
  const bindingLink = path.join(
    repositoryRoot,
    "node_modules",
    ...bindingName.split("/"),
  );
  fs.mkdirSync(path.dirname(bindingLink), { recursive: true });
  removePreviousLink(bindingLink);
  const linkType = process.platform === "win32" ? "junction" : "dir";
  fs.symlinkSync(bindingDirectory, bindingLink, linkType);
}

function removePreviousLink(bindingLink) {
  try {
    const stat = fs.lstatSync(bindingLink);
    if (!stat.isSymbolicLink()) {
      throw new Error(`refusing to replace non-symlinked package at ${bindingLink}`);
    }
    fs.unlinkSync(bindingLink);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
}

function run(command, arguments_, env) {
  const result = spawnSync(command, arguments_, {
    cwd: repositoryRoot,
    env,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
