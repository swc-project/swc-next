#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { bindingFileNames, environmentFiles } from "./wasm-package/files.mjs";
import { prepareBrowserPackage } from "./browser-package.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "..");

const rawArguments = process.argv.slice(2);
if (rawArguments[0] === "--") rawArguments.shift();
const component = rawArguments[0]?.startsWith("-") ? undefined : rawArguments.shift();
const options = parseOptions(rawArguments);
if (component === undefined && options.outputDirectory !== undefined) {
  throw new Error("--output-dir requires a component\n" + usage());
}

const cargoMetadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps", "--locked"], {
    cwd: repositoryRoot,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
  }),
);
const components = discoverComponents(cargoMetadata);
if (component !== undefined && !components.has(component)) {
  throw new Error(usage(components));
}

for (const name of component === undefined ? components.keys() : [component]) {
  buildComponent(name, options);
}

function buildComponent(name, { outputDirectory, release }) {
  const config = components.get(name);
  const names = bindingFileNames(config.binaryName);
  const cargoArguments = [
    "build",
    "--locked",
    "--package",
    config.cargoPackage,
    "--target",
    config.target,
  ];
  if (release) cargoArguments.push("--profile", "wasm-release");
  execFileSync("cargo", cargoArguments, {
    cwd: repositoryRoot,
    stdio: "inherit",
  });

  const destinationDirectory =
    outputDirectory === undefined
      ? path.join(repositoryRoot, "target", "wasm-package", name)
      : path.resolve(process.cwd(), outputDirectory);
  fs.mkdirSync(destinationDirectory, { recursive: true });
  const source = path.join(
    cargoMetadata.target_directory,
    config.target,
    release ? "wasm-release" : "debug",
    config.artifact,
  );
  execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--locked",
      "--package",
      "bridge_cli",
      "--",
      "generate",
      source,
      "--output-dir",
      destinationDirectory,
      "--out-name",
      config.binaryName,
    ],
    { cwd: repositoryRoot, stdio: "inherit" },
  );
  for (const [file, source] of environmentFiles(names, config.version)) {
    // The CLI emits the runtime alongside the bridge and stripped Wasm.
    if (file !== names.runtime) {
      fs.writeFileSync(path.join(destinationDirectory, file), source);
    }
  }

  if (name === "core") prepareBrowserPackage(destinationDirectory);
}

function discoverComponents(metadata) {
  const packagesByManifest = new Map(
    metadata.packages.map((pkg) => [path.resolve(pkg.manifest_path), pkg]),
  );
  const npmDirectory = path.join(repositoryRoot, "npm");
  const components = new Map();
  for (const entry of fs
    .readdirSync(npmDirectory, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .sort((left, right) => left.name.localeCompare(right.name))) {
    const packageJsonPath = path.join(npmDirectory, entry.name, "package.json");
    if (!fs.existsSync(packageJsonPath)) continue;
    const manifest = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
    if (manifest.private === true || manifest.wasm === undefined) continue;

    const wasm = manifest.wasm;
    if (wasm.target !== "wasm32-unknown-unknown") {
      throw new Error(`${packageJsonPath} has unsupported wasm target ${wasm.target}`);
    }
    if (typeof wasm.binaryName !== "string" || wasm.binaryName.length === 0) {
      throw new Error(`${packageJsonPath} has an invalid wasm binaryName`);
    }

    const cargoManifestPath = path.join(npmDirectory, entry.name, "Cargo.toml");
    const cargoPackage = packagesByManifest.get(path.resolve(cargoManifestPath));
    if (cargoPackage === undefined) {
      throw new Error(`missing Cargo package for ${packageJsonPath}`);
    }
    const library = cargoPackage.targets.find((target) => target.kind.includes("cdylib"));
    if (library === undefined) {
      throw new Error(`${cargoManifestPath} does not define a cdylib target`);
    }

    components.set(entry.name, {
      artifact: `${library.name}.wasm`,
      binaryName: wasm.binaryName,
      cargoPackage: cargoPackage.name,
      target: wasm.target,
      version: manifest.version,
    });
  }
  return components;
}

function parseOptions(arguments_) {
  const optionsList = arguments_[0] === "--" ? arguments_.slice(1) : arguments_;
  let outputDirectory;
  let release = false;

  for (let index = 0; index < optionsList.length; index += 1) {
    const option = optionsList[index];
    if (option === "--release" || option === "-r") {
      if (release) throw new Error(usage());
      release = true;
      continue;
    }

    const separator = option.indexOf("=");
    const name = separator === -1 ? option : option.slice(0, separator);
    if (name !== "--output-dir" && name !== "-o") {
      throw new Error(usage());
    }
    if (outputDirectory !== undefined) throw new Error(usage());

    const value =
      separator === -1 ? optionsList[(index += 1)] : option.slice(separator + 1);
    if (typeof value !== "string" || value.length === 0 || value.startsWith("-")) {
      throw new Error(usage());
    }
    outputDirectory = value;
  }

  return { outputDirectory, release };
}

function usage(components) {
  const names = components === undefined ? "<component>" : [...components.keys()].join("|");
  return `usage: build-wasm.mjs [${names}] [--release|-r] [--output-dir <directory>]`;
}
