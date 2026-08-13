#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../..");

function fail(message) {
  throw new Error(`release-version: ${message}`);
}

function readManifest(relativePath) {
  return JSON.parse(
    fs.readFileSync(path.join(repositoryRoot, relativePath), "utf8")
  );
}

function npmManifestPaths() {
  const paths = ["package.json"];
  for (const entry of fs
    .readdirSync(path.join(repositoryRoot, "napi"), { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .sort((left, right) => left.name.localeCompare(right.name))) {
    const manifestPath = `napi/${entry.name}/package.json`;
    if (fs.existsSync(path.join(repositoryRoot, manifestPath))) {
      paths.push(manifestPath);
    }
  }
  return paths;
}

export function npmPackages() {
  return npmManifestPaths()
    .slice(1)
    .flatMap((manifestPath) => {
      const manifest = readManifest(manifestPath);
      if (manifest.private === true || !manifest.napi) return [];
      return [{ directory: path.dirname(manifestPath), manifest }];
    });
}

function writeVersion(version) {
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
    fail(`unsupported version: ${version}`);
  }

  for (const manifestPath of npmManifestPaths()) {
    const manifest = readManifest(manifestPath);
    manifest.version = version;
    fs.writeFileSync(
      path.join(repositoryRoot, manifestPath),
      `${JSON.stringify(manifest, null, 2)}\n`
    );
  }
}

function cargoVersion() {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repositoryRoot,
      encoding: "utf8",
    })
  );
  const versions = new Set(
    metadata.packages
      .filter(({ name }) => name.startsWith("swc_next_"))
      .map(({ version }) => version)
  );
  if (versions.size !== 1) fail("Rust workspace versions are not synchronized");
  return versions.values().next().value;
}

function main() {
  const [command, argument, ...extraArguments] = process.argv.slice(2);
  if (extraArguments.length > 0) fail("too many arguments");

  if (command === "write" && argument) {
    writeVersion(argument);
  } else if (command === "cargo-version" && !argument) {
    console.log(cargoVersion());
  } else if (command === "npm-packages" && !argument) {
    for (const { directory } of npmPackages()) console.log(directory);
  } else {
    fail("usage: version.mjs write <version>|cargo-version|npm-packages");
  }
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  }
}
