import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import { generatedFiles, readContext } from "../common.mjs";

export default function register(cli) {
  cli
    .command("create-npm-dirs", "Create Wasm binding packages")
    .action(createNpmDirs);
}

async function createNpmDirs(userOptions = {}) {
  const context = await readContext(userOptions);
  const files = generatedFiles(context);

  if (!context.options.dryRun) {
    await mkdir(context.bindingDirectory, { recursive: true });
    for (const [name, content] of files) {
      await writeFile(path.join(context.bindingDirectory, name), content);
    }
  }

  return {
    bindingDirectory: context.bindingDirectory,
    files: [...files.keys()],
  };
}
