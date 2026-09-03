#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { jsPackages, npmPackages } from "./version.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../..");

function readManifest(directory) {
  return JSON.parse(
    fs.readFileSync(path.join(directory, "package.json"), "utf8")
  );
}

function checkFiles(directory, manifest) {
  for (const file of manifest.files ?? []) {
    if (!fs.existsSync(path.join(directory, file))) {
      throw new Error(`${manifest.name} is missing ${file}`);
    }
  }
}

function collectPackages(releaseRoot) {
  const bindings = [];
  const apiPackages = [];

  for (const { directory, manifest } of npmPackages()) {
    const apiRoot = path.join(repositoryRoot, directory);
    const apiManifest = readManifest(apiRoot);
    const component = path.basename(directory);
    const bindingsRoot = path.join(releaseRoot, component);
    const expectedNativeBindings = new Map(
      Object.entries(apiManifest.optionalDependencies ?? {})
    );

    if (expectedNativeBindings.size !== manifest.napi.targets.length) {
      throw new Error(`${apiManifest.name} optionalDependencies are incomplete`);
    }
    const expectedBindings = new Map(expectedNativeBindings);
    if (manifest.wasm) {
      expectedBindings.set(manifest.wasm.packageName, apiManifest.version);
    }

    const actualBindings = new Set();
    for (const entry of fs
      .readdirSync(bindingsRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .sort((left, right) => left.name.localeCompare(right.name))) {
      const bindingRoot = path.join(bindingsRoot, entry.name);
      const binding = readManifest(bindingRoot);
      if (!expectedBindings.has(binding.name)) {
        throw new Error(`unexpected binding package: ${binding.name}`);
      }
      if (actualBindings.has(binding.name)) {
        throw new Error(`duplicate binding package: ${binding.name}`);
      }
      if (
        binding.version !== apiManifest.version ||
        expectedBindings.get(binding.name) !== apiManifest.version
      ) {
        throw new Error(`${binding.name} does not match ${apiManifest.version}`);
      }
      actualBindings.add(binding.name);
      checkFiles(bindingRoot, binding);
      bindings.push({ directory: bindingRoot, manifest: binding });
    }

    if (actualBindings.size !== expectedBindings.size) {
      const missing = [...expectedBindings.keys()].filter(
        (name) => !actualBindings.has(name)
      );
      throw new Error(
        `missing ${apiManifest.name} bindings: ${missing.join(", ")}`
      );
    }

    checkFiles(apiRoot, apiManifest);
    apiPackages.push({ directory: apiRoot, manifest: apiManifest });
  }

  const javascript = jsPackages().map(({ directory, manifest }) => {
    const packageRoot = path.join(repositoryRoot, directory);
    checkFiles(packageRoot, manifest);
    return { directory: packageRoot, manifest };
  });
  const versions = new Set(
    [...javascript, ...apiPackages].map(({ manifest }) => manifest.version),
  );
  if (versions.size !== 1) {
    throw new Error("NPM package versions are not synchronized");
  }

  return [...bindings, ...javascript, ...apiPackages];
}

function packageExists({ name, version }) {
  try {
    execFileSync("npm", ["view", `${name}@${version}`, "version"], {
      stdio: "ignore",
    });
    return true;
  } catch {
    return false;
  }
}

function publishPackage({ directory, manifest }, dryRun) {
  const specifier = `${manifest.name}@${manifest.version}`;
  if (packageExists(manifest)) {
    console.log(`Skipping existing ${specifier}.`);
    return;
  }

  const arguments_ = ["publish", "--access", "public", "--no-git-checks"];
  if (dryRun) arguments_.push("--dry-run");
  execFileSync("pnpm", arguments_, { cwd: directory, stdio: "inherit" });
}

function main() {
  const [releaseDirectory, option, ...extraArguments] = process.argv.slice(2);
  const dryRun = option === "--dry-run";
  if (!releaseDirectory || extraArguments.length > 0 || (option && !dryRun)) {
    throw new Error("usage: publish-npm.mjs <release-directory> [--dry-run]");
  }

  const releaseRoot = path.resolve(repositoryRoot, releaseDirectory);
  for (const package_ of collectPackages(releaseRoot)) {
    publishPackage(package_, dryRun);
  }
}

try {
  main();
} catch (error) {
  console.error(
    `release-npm: ${error instanceof Error ? error.message : error}`
  );
  process.exit(1);
}
