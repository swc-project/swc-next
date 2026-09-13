import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { bindingFileNames } from "./wasm-package/files.mjs";

const repositoryRoot = fileURLToPath(new URL("../", import.meta.url));

export function prepareBrowserPackage(sourceDirectory) {
  const packageDirectory = path.join(repositoryRoot, "npm/core-browser");
  const directory = path.join(packageDirectory, "src");
  fs.mkdirSync(directory, { recursive: true });
  const manifest = JSON.parse(
    fs.readFileSync(path.join(packageDirectory, "package.json"), "utf8"),
  );
  const coreManifest = JSON.parse(
    fs.readFileSync(path.join(repositoryRoot, "npm/core/package.json"), "utf8"),
  );
  const names = bindingFileNames(coreManifest.wasm.binaryName);

  if (manifest.version !== coreManifest.version) {
    throw new Error("browser and core package versions must match");
  }

  for (const name of [names.wasm, names.bridge, names.browser, names.runtime]) {
    const source = path.resolve(sourceDirectory, name);
    const destination = path.join(directory, name);
    if (source !== destination) fs.copyFileSync(source, destination);
  }

  const declaration = fs.readFileSync(
    path.join(repositoryRoot, "npm/core/src-js/binding.d.ts"),
    "utf8",
  );
  fs.writeFileSync(
    path.join(directory, "binding.d.ts"),
    declaration.replaceAll("Buffer", "Uint8Array"),
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.length !== 3) {
    throw new Error("usage: browser-package.mjs <source-directory>");
  }
  prepareBrowserPackage(process.argv[2]);
}
