#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { bindingFileNames, environmentFiles } from "./wasm-package/files.mjs";

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
  execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
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
  const metadataPackages = findMetadataPackages(cargoMetadata, config.cargoPackage);
  const metadataDirectory = path.join(
    cargoMetadata.target_directory,
    "swc-next-bridge",
    "metadata",
    name,
  );
  fs.rmSync(metadataDirectory, { force: true, recursive: true });
  fs.mkdirSync(metadataDirectory, { recursive: true });

  const cargoEnvironment = {
    ...process.env,
    SWC_BRIDGE_METADATA_DIR: metadataDirectory,
  };
  for (const packageName of metadataPackages) {
    cargoEnvironment[forceBuildEnvironment(packageName)] = randomUUID();
  }

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
    env: cargoEnvironment,
    stdio: "inherit",
  });

  const destinationDirectory =
    outputDirectory === undefined
      ? path.join(repositoryRoot, "target", "wasm-package", name)
      : path.resolve(process.cwd(), outputDirectory);
  fs.mkdirSync(destinationDirectory, { recursive: true });
  const bridgeOutput = path.join(destinationDirectory, names.bridge);
  execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--locked",
      "--package",
      "bridge_codegen",
      "--bin",
      "swc-next-bridge-codegen",
      "--",
      "generate",
      "--metadata-dir",
      metadataDirectory,
      "--runtime",
      names.runtime,
      "--output",
      bridgeOutput,
      ...metadataPackages.flatMap((packageName) => ["--package", packageName]),
    ],
    { cwd: repositoryRoot, stdio: "inherit" },
  );
  for (const [file, source] of environmentFiles(names, config.version)) {
    fs.writeFileSync(path.join(destinationDirectory, file), source);
  }
  if (name === "core") {
    const declarationSource = path.join(repositoryRoot, "npm/core/src-js/binding.d.ts");
    const declarationOutput = path.join(
      repositoryRoot,
      "npm/core-wasm/binding.d.ts",
    );
    const declaration = fs.readFileSync(declarationSource, "utf8");
    fs.mkdirSync(path.dirname(declarationOutput), { recursive: true });
    fs.writeFileSync(
      declarationOutput,
      declaration.replaceAll("Buffer", "Uint8Array"),
    );
  }

  const source = path.join(
    cargoMetadata.target_directory,
    config.target,
    release ? "wasm-release" : "debug",
    config.artifact,
  );
  fs.copyFileSync(source, path.join(destinationDirectory, names.wasm));
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

function findMetadataPackages(metadata, rootPackageName) {
  const packagesById = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
  const rootPackage = metadata.packages.find((pkg) => pkg.name === rootPackageName);
  if (rootPackage === undefined) {
    throw new Error(`missing Cargo package ${rootPackageName}`);
  }
  const nodesById = new Map(
    (metadata.resolve?.nodes ?? []).map((node) => [node.id, node]),
  );
  const dependencyIds = new Set();
  const pending = [rootPackage.id];
  while (pending.length !== 0) {
    const packageId = pending.pop();
    if (dependencyIds.has(packageId)) continue;
    dependencyIds.add(packageId);
    for (const dependency of nodesById.get(packageId)?.deps ?? []) {
      pending.push(dependency.pkg);
    }
  }

  const macroPackageIds = new Set(
    metadata.packages
      .filter((pkg) => pkg.name === "swc_next_bridge_macros")
      .map((pkg) => pkg.id),
  );
  const workspacePackageIds = new Set(metadata.workspace_members);
  const packageNames = new Set([rootPackageName]);
  for (const packageId of dependencyIds) {
    if (!workspacePackageIds.has(packageId)) continue;
    const node = nodesById.get(packageId);
    if (node?.deps.some((dependency) => macroPackageIds.has(dependency.pkg))) {
      packageNames.add(packagesById.get(packageId).name);
    }
  }
  return [...packageNames].sort();
}

function forceBuildEnvironment(packageName) {
  return `SWC_BRIDGE_FORCE_BUILD_${packageName.replaceAll("-", "_").toUpperCase()}`;
}

function usage(components) {
  const names = components === undefined ? "<component>" : [...components.keys()].join("|");
  return `usage: build-wasm.mjs [${names}] [--release|-r] [--output-dir <directory>]`;
}
