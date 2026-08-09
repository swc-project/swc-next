import fs from "node:fs";
import path from "node:path";

import { collectCompilerManifest } from "__TS_ROOT__/src/testRunner/compilerRunner.ts";

const TYPESCRIPT_REVISION = process.env.TS_TYPESCRIPT_REVISION!;
const fixtureRoot = process.env.TS_FIXTURE_ROOT!;
const outputPath = process.env.TS_MANIFEST_OUTPUT!;
if (!TYPESCRIPT_REVISION) throw new Error("missing TypeScript revision");
process.chdir(fixtureRoot);

const roots = ["tests/cases/compiler", "tests/cases/conformance"];
const files: string[] = [];

function visit(directory: string) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
        const entryPath = path.posix.join(directory, entry.name);
        if (entry.isDirectory()) {
            visit(entryPath);
        }
        else if (/\.tsx?$/.test(entry.name)) {
            files.push(entryPath);
        }
    }
}

for (const root of roots) visit(root);
files.sort();

const output = fs.openSync(outputPath, "w");
const write = (value: unknown) => fs.writeSync(output, JSON.stringify(value) + "\n");

write({
    schema: 1,
    typescriptRevision: TYPESCRIPT_REVISION,
    variantColumns: [
        "test",
        "configuration",
        "configuredName",
        "globalDiagnosticCodes",
        "units",
    ],
    unitColumns: [
        "unit",
        "occurrence",
        "status",
        "contentByteLength",
        "contentHash",
        "diagnosticCodes",
        "checkJs",
        "semanticChecked",
        "externalModuleIndicator",
        "commonJsModuleIndicator",
        "impliedNodeFormat",
        "sourceType",
        "redirect",
    ],
    enums: {
        status: { e: "exact", r: "redirect", a: "absent" },
        externalModuleIndicator: { n: "none", s: "syntax", f: "forced" },
        impliedNodeFormat: { n: "none", c: "commonjs", m: "module" },
        sourceType: { s: "script", c: "commonjs", m: "module" },
        redirect: { p: "package", s: "shadowed", i: "identity" },
    },
});

for (let index = 0; index < files.length; index++) {
    const file = files[index];
    const test = file.slice("tests/cases/".length);
    for (const variant of collectCompilerManifest(file)) {
        const configuration = Object.entries(variant.configuration)
            .sort(([left], [right]) => left.localeCompare(right));
        const units = variant.units.map(unit => [
            unit.name,
            unit.occurrence,
            unit.status,
            unit.contentByteLength,
            unit.contentHash,
            unit.diagnosticCodes,
            unit.checkJs,
            unit.semanticChecked,
            unit.externalModuleIndicator,
            unit.commonJsModuleIndicator,
            unit.impliedNodeFormat,
            unit.sourceType,
            unit.redirect,
        ]);
        write([test, configuration, variant.name, variant.globalDiagnosticCodes, units]);
    }
    if ((index + 1) % 250 === 0) {
        console.error(`${index + 1}/${files.length}`);
    }
}

fs.closeSync(output);
