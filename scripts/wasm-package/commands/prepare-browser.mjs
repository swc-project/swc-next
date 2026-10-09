import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { bindingFileNames } from "../common.mjs";
import { REPOSITORY_ROOT } from "../constants.mjs";

export default function register(cli) {
  cli
    .command("prepare-browser", "Assemble the browser package")
    .option("--source-dir <directory>", "Browser binding source directory")
    .action(prepareBrowserPackage);
}

async function prepareBrowserPackage({
  sourceDir,
  cwd = process.cwd(),
  dryRun = false,
} = {}) {
  if (!sourceDir) throw new Error("prepare-browser requires --source-dir");
  if (dryRun) throw new Error("prepare-browser does not support --dry-run");
  const sourceDirectory = path.resolve(cwd, sourceDir);
  const packageDirectory = path.join(REPOSITORY_ROOT, "npm/core-browser");
  const directory = path.join(packageDirectory, "src");
  await mkdir(directory, { recursive: true });
  const manifest = JSON.parse(
    await readFile(path.join(packageDirectory, "package.json"), "utf8"),
  );
  const coreManifest = JSON.parse(
    await readFile(path.join(REPOSITORY_ROOT, "npm/core/package.json"), "utf8"),
  );
  const names = bindingFileNames(coreManifest.wasm.binaryName);

  if (manifest.version !== coreManifest.version) {
    throw new Error("browser and core package versions must match");
  }

  for (const name of [names.wasm, names.bridge, names.browser, names.runtime]) {
    const source = path.resolve(sourceDirectory, name);
    const destination = path.join(directory, name);
    if (source !== destination) await copyFile(source, destination);
  }

  for (const name of [
    "ast.js",
    "ast.d.ts",
    "visitor-keys.js",
    "child-visitors.js",
    "visitor-keys.d.ts",
    "transform.js",
  ]) {
    await copyFile(
      path.join(REPOSITORY_ROOT, "npm/core/src-js", name),
      path.join(directory, name),
    );
  }

  const declaration = await readFile(
    path.join(REPOSITORY_ROOT, "npm/core/src-js/binding.d.ts"),
    "utf8",
  );
  await writeFile(
    path.join(directory, "binding.d.ts"),
    declaration.replaceAll("Buffer", "Uint8Array"),
  );
}
