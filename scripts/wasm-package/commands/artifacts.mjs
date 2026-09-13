import { copyFile, readdir } from "node:fs/promises";
import path from "node:path";

import { fail, readContext } from "../common.mjs";

export default function register(cli) {
  cli
    .command("artifacts", "Collect Wasm build artifacts")
    .option("-o, -d, --output-dir <directory>", "Build artifacts directory")
    .action(collectArtifacts);
}

async function collectArtifacts(userOptions = {}) {
  const context = await readContext(userOptions);
  if (context.options.dryRun) fail("artifacts does not support --dry-run");

  const outputDirectory = path.resolve(
    context.options.cwd,
    context.options.outputDir,
  );
  const artifactNames = Object.values(context.names);
  const copied = [];
  for (const artifactName of artifactNames) {
    const matches = await findArtifacts(outputDirectory, artifactName);
    if (matches.length === 0)
      fail(`missing WebAssembly artifact: ${artifactName}`);
    if (matches.length > 1) {
      fail(
        `duplicate WebAssembly artifact ${artifactName}: ${matches.join(", ")}`,
      );
    }
    const destination = path.join(context.bindingDirectory, artifactName);
    await copyFile(matches[0], destination);
    copied.push({ destination, source: matches[0] });
  }
  return { artifacts: copied };
}

async function findArtifacts(directory, artifactName, matches = []) {
  let entries;
  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch (error) {
    fail(`cannot read artifact directory ${directory}: ${error.message}`);
  }
  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory() && entry.name !== "node_modules") {
      await findArtifacts(entryPath, artifactName, matches);
    } else if (entry.isFile() && entry.name === artifactName) {
      matches.push(entryPath);
    }
  }
  return matches;
}
