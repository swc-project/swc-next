import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { bindingFileNames, environmentFiles } from "./wasm-package/files.mjs";

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

  for (const name of [names.wasm, names.bridge]) {
    const source = path.resolve(sourceDirectory, name);
    const destination = path.join(directory, name);
    if (source !== destination) fs.copyFileSync(source, destination);
  }
  const files = environmentFiles(names, manifest.version);
  for (const name of [names.browser, names.runtime]) {
    fs.writeFileSync(path.join(directory, name), files.get(name));
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
