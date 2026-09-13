import { access, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { fail, generatedFiles, readContext, readJson } from "../common.mjs";

export default function register(cli) {
  cli
    .command("pre-publish", "Validate and prepare Wasm packages for publishing")
    .alias("prepublish")
    .option("--no-gh-release", "Disable GitHub releases")
    .option("--skip-optional-publish", "Skip publishing optional packages")
    .option("-t, --tagstyle, --tag-style <style>", "Release tag style")
    .action(prePublish);
}

async function prePublish(userOptions = {}) {
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

  await validatePackage(
    context,
    generatedFiles(context),
    context.options.dryRun,
  );

  if (!context.options.dryRun) {
    await writeFile(
      context.packageJsonPath,
      `${JSON.stringify(packageJson, null, 2)}\n`,
    );
  }

  return { bindingDirectory: context.bindingDirectory, packageJson };
}

async function validatePackage(context, generated, dryRun) {
  if (dryRun) return;

  const manifestPath = path.join(context.bindingDirectory, "package.json");
  const manifest = await readJson(manifestPath, "generated package.json");
  if (manifest.name !== context.config.packageName) {
    fail(`generated package has unexpected name: ${manifest.name}`);
  }
  if (manifest.version !== context.packageJson.version) {
    fail(
      `${manifest.name} version ${manifest.version} does not match ${context.packageJson.version}`,
    );
  }
  for (const file of manifest.files ?? []) {
    try {
      await access(path.join(context.bindingDirectory, file));
    } catch {
      fail(`${manifest.name} is missing ${file}`);
    }
  }
  for (const name of generated.keys()) {
    const actual = await readFile(
      path.join(context.bindingDirectory, name),
      "utf8",
    );
    if (actual !== generated.get(name)) {
      fail(`${manifest.name} contains a stale generated file: ${name}`);
    }
  }
  for (const name of [
    context.names.node,
    context.names.browser,
    context.names.module,
  ]) {
    const source = await readFile(
      path.join(context.bindingDirectory, name),
      "utf8",
    );
    if (
      !source.includes(
        `\nexport const version = ${JSON.stringify(context.packageJson.version)};\n`,
      )
    ) {
      fail(`${context.packageJson.name} contains a stale entry point: ${name}`);
    }
  }
}
