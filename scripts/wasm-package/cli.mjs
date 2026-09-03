#!/usr/bin/env node

import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { collectArtifacts, createNpmDirs, prePublish } from "./index.mjs";

const argumentAliases = new Map([
  ["-d", "-o"],
  ["--tagstyle", "--tag-style"],
]);
const argumentOptions = {
  "config-path": { type: "string", short: "c" },
  cwd: { type: "string" },
  "dry-run": { type: "boolean" },
  "no-gh-release": { type: "boolean" },
  "npm-dir": { type: "string", short: "p" },
  "output-dir": { type: "string", short: "o" },
  "package-json-path": { type: "string" },
  "skip-optional-publish": { type: "boolean" },
  "tag-style": { type: "string", short: "t" },
};

export async function run(arguments_) {
  const [command, ...rawOptions] = arguments_;
  const normalizedOptions = rawOptions.map((argument) => {
    const separator = argument.indexOf("=");
    const name = separator === -1 ? argument : argument.slice(0, separator);
    const alias = argumentAliases.get(name);
    return alias === undefined ? argument : alias + argument.slice(name.length);
  });
  const { values } = parseArgs({
    args: normalizedOptions,
    options: argumentOptions,
  });
  const options = Object.fromEntries(
    [
      ["configPath", values["config-path"]],
      ["cwd", values.cwd],
      ["dryRun", values["dry-run"]],
      ["npmDir", values["npm-dir"]],
      ["outputDir", values["output-dir"]],
      ["packageJsonPath", values["package-json-path"]],
      ["skipOptionalPublish", values["skip-optional-publish"]],
      ["tagStyle", values["tag-style"]],
    ].filter(([, value]) => value !== undefined),
  );
  if (values["no-gh-release"]) options.ghRelease = false;

  if (command === "create-npm-dirs") return createNpmDirs(options);
  if (command === "artifacts") return collectArtifacts(options);
  if (command === "pre-publish" || command === "prepublish") {
    return prePublish(options);
  }
  throw new Error(usage());
}

function usage() {
  return "usage: wasm-package <create-npm-dirs|artifacts|pre-publish> [options]";
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    await run(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
