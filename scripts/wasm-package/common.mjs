import { readFile } from "node:fs/promises";
import path from "node:path";

import { DEFAULT_OPTIONS, TARGET } from "./constants.mjs";

export function bindingFileNames(binaryName) {
  return {
    wasm: `${binaryName}.wasm`,
    bridge: `${binaryName}.bridge.mjs`,
    node: `${binaryName}.node.mjs`,
    browser: `${binaryName}.browser.mjs`,
    module: `${binaryName}.module.mjs`,
    runtime: `${binaryName}.runtime.mjs`,
  };
}

export async function readContext(userOptions) {
  const options = {
    cwd: process.cwd(),
    ...DEFAULT_OPTIONS,
    ...userOptions,
  };
  const packageJsonPath = path.resolve(options.cwd, options.packageJsonPath);
  const packageJson = await readJson(packageJsonPath, "package.json");
  const config = options.configPath
    ? await readJson(
        path.resolve(options.cwd, options.configPath),
        "WebAssembly package config",
      )
    : packageJson.wasm;

  validateConfig(packageJson, config);
  if (options.tagStyle !== "npm" && options.tagStyle !== "lerna") {
    fail(`unsupported tag style: ${options.tagStyle}`);
  }

  const bindingDirectory = path.resolve(options.cwd, options.npmDir, TARGET);
  return {
    bindingDirectory,
    config,
    names: bindingFileNames(config.binaryName),
    options,
    packageJson,
    packageJsonPath,
  };
}

function validateConfig(packageJson, config) {
  if (!packageJson || typeof packageJson !== "object") {
    fail("package.json must contain an object");
  }
  if (typeof packageJson.name !== "string" || packageJson.name.length === 0) {
    fail("package.json must contain a package name");
  }
  if (
    typeof packageJson.version !== "string" ||
    packageJson.version.length === 0
  ) {
    fail("package.json must contain a package version");
  }
  if (!config || typeof config !== "object" || Array.isArray(config)) {
    fail("package.json must contain a wasm configuration");
  }
  if (
    typeof config.binaryName !== "string" ||
    !/^[A-Za-z0-9_-]+$/.test(config.binaryName)
  ) {
    fail("wasm.binaryName must be a non-empty file name");
  }
  if (
    typeof config.packageName !== "string" ||
    config.packageName.length === 0
  ) {
    fail("wasm.packageName must be a non-empty package name");
  }
}

export function generatedFiles(context) {
  const manifest = bindingManifest(context);
  return new Map([
    ["package.json", `${JSON.stringify(manifest, null, 2)}\n`],
    ["README.md", bindingReadme(context)],
  ]);
}

function bindingManifest(context) {
  const packageJson = context.packageJson;
  return removeUndefined({
    name: context.config.packageName,
    version: packageJson.version,
    description: packageJson.description,
    type: "module",
    main: `./${context.names.node}`,
    exports: {
      ".": {
        unpkg: `./${context.names.browser}`,
        webpack: `./${context.names.module}`,
        browser: `./${context.names.browser}`,
        node: `./${context.names.node}`,
        default: `./${context.names.module}`,
      },
      "./package.json": "./package.json",
    },
    files: [
      context.names.bridge,
      context.names.node,
      context.names.browser,
      context.names.module,
      context.names.runtime,
      context.names.wasm,
    ],
    keywords: packageJson.keywords,
    author: packageJson.author,
    authors: packageJson.authors,
    homepage: packageJson.homepage,
    license: packageJson.license,
    repository: packageJson.repository,
    bugs: packageJson.bugs,
    publishConfig: packageJson.publishConfig,
  });
}

function bindingReadme(context) {
  return `# \`${context.config.packageName}\`\n\nThis is the ${TARGET} binding for \`${context.packageJson.name}\`.\n`;
}

export async function readJson(file, description) {
  let content;
  try {
    content = await readFile(file, "utf8");
  } catch (error) {
    fail(`cannot read ${description} at ${file}: ${error.message}`);
  }
  try {
    return JSON.parse(content);
  } catch (error) {
    fail(`cannot parse ${description} at ${file}: ${error.message}`);
  }
}

function removeUndefined(object) {
  return Object.fromEntries(
    Object.entries(object).filter(([, value]) => value !== undefined),
  );
}

export function fail(message) {
  throw new Error(`wasm-package: ${message}`);
}
