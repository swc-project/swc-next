import { execFileSync } from "node:child_process";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const components = ["core", "parser", "codegen"];
const target = "x86_64-unknown-linux-gnu";

export function measureBinarySizes(root, revision, rustc) {
  return {
    schemaVersion: 1,
    revision,
    target,
    profile: "release",
    rustc,
    artifacts: components.map((component) => {
      const file = `npm/${component}/src-js/swc-next-${component}.linux-x64-gnu.node`;
      const stat = statSync(resolve(root, file));
      if (!stat.isFile() || stat.size === 0) {
        throw new Error(`Missing or empty binding: ${file}`);
      }
      return { name: `@swc-next/${component}`, file, bytes: stat.size };
    }),
  };
}

function validateMeasurement(measurement) {
  if (
    measurement.schemaVersion !== 1 ||
    !/^[a-f0-9]{40}$/.test(measurement.revision) ||
    measurement.target !== target ||
    measurement.profile !== "release" ||
    typeof measurement.rustc !== "string" ||
    !measurement.rustc.startsWith("rustc ") ||
    measurement.artifacts?.length !== components.length
  ) {
    throw new Error("Invalid binary size measurement");
  }
  for (const component of components) {
    const matches = measurement.artifacts.filter(
      (artifact) => artifact.name === `@swc-next/${component}`,
    );
    if (
      matches.length !== 1 ||
      !Number.isSafeInteger(matches[0].bytes) ||
      matches[0].bytes <= 0
    ) {
      throw new Error(`Invalid binary size for ${component}`);
    }
  }
}

function formatSize(bytes) {
  return `${(bytes / 1024).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })} KiB`;
}

export function renderBinarySizeDiff(base, merge) {
  validateMeasurement(base);
  validateMeasurement(merge);
  if (base.rustc !== merge.rustc) {
    throw new Error(
      "Cannot compare binaries built with different Rust toolchains",
    );
  }
  const lines = [
    "### Binary size diff",
    "",
    `Release Node.js bindings for \`${target}\`, stripped by the release build.`,
    "Sizes are uncompressed file sizes; differences use exact bytes. Reporting only, with no failure threshold.",
    "",
    `Merge: \`${merge.revision}\`; base: \`${base.revision}\`.`,
    `Toolchain: \`${merge.rustc}\`.`,
    "",
    "| Artifact | Merge | Base | Change (bytes) | Change (%) |",
    "| --- | ---: | ---: | ---: | ---: |",
  ];
  for (const component of components) {
    const name = `@swc-next/${component}`;
    const before = base.artifacts.find(
      (artifact) => artifact.name === name,
    ).bytes;
    const after = merge.artifacts.find(
      (artifact) => artifact.name === name,
    ).bytes;
    const delta = after - before;
    const sign = delta > 0 ? "+" : "";
    const status = delta > 0 ? "🔴" : delta < 0 ? "🟢" : "⚪";
    const percentage = ((delta / before) * 100).toFixed(3);
    lines.push(
      `| \`${name}\` | ${formatSize(after)} | ${formatSize(before)} | ${sign}${delta.toLocaleString("en-US")} | ${sign}${percentage}% ${status} |`,
    );
  }
  return `${lines.join("\n")}\n`;
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const [command, first, second] = process.argv.slice(2);
  if (command === "measure" && first && second) {
    const rustc = execFileSync("rustc", ["--version"], {
      encoding: "utf8",
    }).trim();
    const measurement = measureBinarySizes(process.cwd(), second, rustc);
    validateMeasurement(measurement);
    writeFileSync(first, `${JSON.stringify(measurement, null, 2)}\n`);
  } else if (command === "compare" && first && second) {
    process.stdout.write(
      renderBinarySizeDiff(
        JSON.parse(readFileSync(first, "utf8")),
        JSON.parse(readFileSync(second, "utf8")),
      ),
    );
  } else {
    throw new Error(
      "Usage: binary-size.mjs measure <output.json> <sha> | compare <base.json> <merge.json>",
    );
  }
}
