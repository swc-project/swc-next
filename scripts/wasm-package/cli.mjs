#!/usr/bin/env node

import process from "node:process";
import { fileURLToPath } from "node:url";
import { cac } from "cac";

import artifacts from "./commands/artifacts.mjs";
import createNpmDirs from "./commands/create-npm-dirs.mjs";
import prepareBrowser from "./commands/prepare-browser.mjs";
import prePublish from "./commands/pre-publish.mjs";

async function run(arguments_) {
  const cli = cac("wasm-package");
  cli
    .option("-c, --config-path <path>", "WebAssembly package configuration")
    .option("--cwd <directory>", "Working directory")
    .option("--dry-run", "Preview without writing files")
    .option("-p, --npm-dir <directory>", "Binding package directory")
    .option("--package-json-path <path>", "Package manifest path");

  createNpmDirs(cli);
  artifacts(cli);
  prePublish(cli);
  prepareBrowser(cli);
  cli.help();
  cli.parse([process.execPath, "wasm-package", ...arguments_], { run: false });
  if (cli.options.help) return;
  if (!cli.matchedCommand) {
    throw new Error(
      `Unknown command: ${cli.args[0] ?? "(none)"}. Run wasm-package --help for usage.`,
    );
  }
  return cli.runMatchedCommand();
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    await run(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
