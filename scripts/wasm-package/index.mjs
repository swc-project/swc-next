import { access, copyFile, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

import { bindingFileNames, environmentFiles } from "./files.mjs";

const SUPPORTED_TARGET = "wasm32-unknown-unknown";

export async function createNpmDirs(userOptions = {}) {
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

export async function collectArtifacts(userOptions = {}) {
  const context = await readContext(userOptions);
  if (context.options.dryRun) fail("artifacts does not support --dry-run");

  const outputDirectory = path.resolve(context.options.cwd, context.options.outputDir);
  const artifactNames = [context.names.wasm, context.names.bridge];
  const copied = [];
  for (const artifactName of artifactNames) {
    const matches = await findArtifacts(outputDirectory, artifactName);
    if (matches.length === 0) fail(`missing WebAssembly artifact: ${artifactName}`);
    if (matches.length > 1) {
      fail(`duplicate WebAssembly artifact ${artifactName}: ${matches.join(", ")}`);
    }
    const destination = path.join(context.bindingDirectory, artifactName);
    await copyFile(matches[0], destination);
    copied.push({ destination, source: matches[0] });
  }
  return { artifacts: copied };
}

export async function prePublish(userOptions = {}) {
  const context = await readContext(userOptions);
  if (context.options.ghRelease) {
    fail("GitHub releases are not supported; pass --no-gh-release");
  }
  if (!context.options.skipOptionalPublish) {
    fail("pass --skip-optional-publish when assembling Wasm bindings");
  }

  const packageJson = structuredClone(context.packageJson);
  packageJson.peerDependencies = {
    ...(packageJson.peerDependencies ?? {}),
    [context.config.packageName]: packageJson.version,
  };
  packageJson.peerDependenciesMeta = {
    ...(packageJson.peerDependenciesMeta ?? {}),
    [context.config.packageName]: {
      ...(packageJson.peerDependenciesMeta?.[context.config.packageName] ?? {}),
      optional: true,
    },
  };

  await validatePackage(context, generatedFiles(context), context.options.dryRun);

  if (!context.options.dryRun) {
    await writeFile(
      context.packageJsonPath,
      `${JSON.stringify(packageJson, null, 2)}\n`,
    );
  }

  return { bindingDirectory: context.bindingDirectory, packageJson };
}

async function readContext(userOptions) {
  const options = {
    cwd: process.cwd(),
    packageJsonPath: "package.json",
    npmDir: "npm",
    outputDir: "./artifacts",
    dryRun: false,
    ghRelease: true,
    skipOptionalPublish: false,
    tagStyle: "lerna",
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

  const bindingDirectory = path.resolve(options.cwd, options.npmDir, config.target);
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
  if (typeof packageJson.version !== "string" || packageJson.version.length === 0) {
    fail("package.json must contain a package version");
  }
  if (!config || typeof config !== "object" || Array.isArray(config)) {
    fail("package.json must contain a wasm configuration");
  }
  if (config.target !== SUPPORTED_TARGET) {
    fail(`only ${SUPPORTED_TARGET} is supported`);
  }
  if (
    typeof config.binaryName !== "string" ||
    !/^[A-Za-z0-9_-]+$/.test(config.binaryName)
  ) {
    fail("wasm.binaryName must be a non-empty file name");
  }
  if (typeof config.packageName !== "string" || config.packageName.length === 0) {
    fail("wasm.packageName must be a non-empty package name");
  }
}

function generatedFiles(context) {
  const manifest = bindingManifest(context);
  return new Map([
    ["package.json", `${JSON.stringify(manifest, null, 2)}\n`],
    ["README.md", bindingReadme(context)],
    ...environmentFiles(context.names, context.packageJson.version),
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
  return `# \`${context.config.packageName}\`\n\nThis is the ${context.config.target} binding for \`${context.packageJson.name}\`.\n`;
}

async function validatePackage(context, generated, dryRun) {
  if (dryRun) return;

  const manifestPath = path.join(context.bindingDirectory, "package.json");
  const manifest = await readJson(manifestPath, "generated package.json");
  if (manifest.name !== context.config.packageName) {
    fail(`generated package has unexpected name: ${manifest.name}`);
  }
  if (manifest.version !== context.packageJson.version) {
    fail(`${manifest.name} version ${manifest.version} does not match ${context.packageJson.version}`);
  }
  for (const file of manifest.files ?? []) {
    try {
      await access(path.join(context.bindingDirectory, file));
    } catch {
      fail(`${manifest.name} is missing ${file}`);
    }
  }
  for (const name of generated.keys()) {
    const actual = await readFile(path.join(context.bindingDirectory, name), "utf8");
    if (actual !== generated.get(name)) {
      fail(`${manifest.name} contains a stale generated file: ${name}`);
    }
  }
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

async function readJson(file, description) {
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
  return Object.fromEntries(Object.entries(object).filter(([, value]) => value !== undefined));
}

function fail(message) {
  throw new Error(`wasm-package: ${message}`);
}
