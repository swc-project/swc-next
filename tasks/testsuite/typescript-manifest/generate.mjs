import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

const scriptRoot = path.dirname(fileURLToPath(import.meta.url));
const fixtureRoot = fs.realpathSync(path.join(scriptRoot, "../fixtures/typescript"));
const manifestPath = path.join(scriptRoot, "../src/cases/typescript/program_manifest.ndjson");
const mode = process.argv[2];

if (mode !== "--write" && mode !== "--verify") {
    throw new Error("usage: node generate.mjs --write|--verify");
}

const cloneScriptPath = path.join(scriptRoot, "../../../scripts/clone_fixtures.sh");
const cloneScript = fs.readFileSync(cloneScriptPath, "utf8");
const revisionMatch = cloneScript.match(
    /clone_repo "microsoft\/TypeScript" "tasks\/testsuite\/fixtures\/typescript" "([0-9a-f]{40})" "typescript"/,
);
if (!revisionMatch) {
    throw new Error(`could not read the TypeScript revision from ${cloneScriptPath}`);
}
const typescriptRevision = revisionMatch[1];

const actualRevision = execFileSync("git", ["-C", fixtureRoot, "rev-parse", "HEAD"], {
    encoding: "utf8",
}).trim();
if (actualRevision !== typescriptRevision) {
    throw new Error(`expected TypeScript ${typescriptRevision}, found ${actualRevision}`);
}

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "swc-next-typescript-manifest-"));
const outputPath = path.join(tempRoot, "program_manifest.ndjson");

try {
    prepareTypeScriptLibraries(tempRoot);
    await buildGenerator(tempRoot);

    const result = spawnSync(process.execPath, [path.join(tempRoot, "generator.cjs")], {
        cwd: fixtureRoot,
        env: {
            ...process.env,
            TS_FIXTURE_ROOT: fixtureRoot,
            TS_MANIFEST_OUTPUT: outputPath,
            TS_MANIFEST_TEMP_ROOT: tempRoot,
            TS_TYPESCRIPT_REVISION: typescriptRevision,
        },
        stdio: "inherit",
    });
    if (result.status !== 0) {
        throw new Error(`manifest generator exited with status ${result.status}`);
    }

    const generated = fs.readFileSync(outputPath);
    if (mode === "--write") {
        const temporaryManifest = `${manifestPath}.tmp`;
        fs.writeFileSync(temporaryManifest, generated);
        fs.renameSync(temporaryManifest, manifestPath);
    }
    else {
        const checkedIn = fs.readFileSync(manifestPath);
        if (!generated.equals(checkedIn)) {
            throw new Error(
                "TypeScript Program manifest is stale; run `pnpm --filter @swc-next/typescript-program-manifest generate`",
            );
        }
        console.error("TypeScript Program manifest is up to date");
    }
}
finally {
    fs.rmSync(tempRoot, { recursive: true, force: true });
}

function prepareTypeScriptLibraries(tempRoot) {
    const builtLocal = path.join(tempRoot, "built/local");
    fs.mkdirSync(builtLocal, { recursive: true });
    const copyright = fs.readFileSync(path.join(fixtureRoot, "scripts/CopyrightNotice.txt"), "utf8")
        .replaceAll("\r\n", "\n");
    const libraries = Function(
        `"use strict"; return (${fs.readFileSync(path.join(fixtureRoot, "src/lib/libs.json"), "utf8")})`,
    )();
    for (const library of libraries.libs) {
        const source = path.join(fixtureRoot, "src/lib", `${library}.d.ts`);
        const targetName = libraries.paths?.[library] ?? `lib.${library}.d.ts`;
        const contents = fs.readFileSync(source, "utf8").replaceAll("\r\n", "\n");
        fs.writeFileSync(path.join(builtLocal, targetName), `${copyright}\n${contents}`);
    }

    const testLib = path.join(tempRoot, "tests/lib");
    fs.mkdirSync(path.dirname(testLib), { recursive: true });
    fs.symlinkSync(path.join(fixtureRoot, "tests/lib"), testLib, "dir");
}

function diagnosticModule() {
    const diagnosticJson = path.join(fixtureRoot, "src/compiler/diagnosticMessages.json");
    const compilerTypes = path.join(fixtureRoot, "src/compiler/types.ts").replaceAll("\\", "/");
    const diagnostics = JSON.parse(fs.readFileSync(diagnosticJson, "utf8"));
    const lines = [
        `import { DiagnosticCategory } from ${JSON.stringify(compilerTypes)};`,
        "function diag(code, category, key, message, reportsUnnecessary, elidedInCompatabilityPyramid, reportsDeprecated) { return { code, category, key, message, reportsUnnecessary, elidedInCompatabilityPyramid, reportsDeprecated }; }",
        "export const Diagnostics = {",
    ];
    for (const [message, details] of Object.entries(diagnostics)) {
        let property = message.split("").map(character => character === "*" ? "_Asterisk"
            : character === "/" ? "_Slash"
            : character === ":" ? "_Colon"
            : /\w/.test(character) ? character
            : "_").join("");
        property = property.replace(/_+/g, "_").replace(/^_(\D)/, "$1").replace(/_$/, "");
        const key = `${property.slice(0, 100)}_${details.code}`;
        lines.push(`${JSON.stringify(property)}: diag(${details.code}, DiagnosticCategory.${details.category}, ${JSON.stringify(key)}, ${JSON.stringify(message)}, ${details.reportsUnnecessary ? "{}" : "undefined"}, ${details.elidedInCompatabilityPyramid ? "true" : "undefined"}, ${details.reportsDeprecated ? "{}" : "undefined"}),`);
    }
    lines.push("};");
    return lines.join("\n");
}

async function buildGenerator(tempRoot) {
    const compilerRunner = path.join(fixtureRoot, "src/testRunner/compilerRunner.ts");
    const testRunnerHarness = path.join(fixtureRoot, "src/testRunner/_namespaces/Harness.ts");
    const harnessIo = path.join(fixtureRoot, "src/harness/harnessIO.ts");
    const injectMarker = "        this.options = this.result.options;";
    const methodMarker = "    private createHarnessTestFile(unit: TestCaseParser.TestUnitData): Compiler.TestFile {";
    const helper = `

    public manifest(units: readonly TestCaseParser.TestUnitData[]) {
        const program = this.result.program;
        const diagnostics = this.result.diagnostics;
        const options = this.result.options;
        const contentIdentity = (text: string) => {
            const bytes = Buffer.from(text, "utf8");
            let high = 0xcbf29ce4;
            let low = 0x84222325;
            for (const byte of bytes) {
                low = (low ^ byte) >>> 0;
                const previousLow = low;
                const lowProduct = previousLow * 0x1b3;
                low = lowProduct >>> 0;
                const carry = Math.floor(lowProduct / 0x100000000);
                high = (Math.imul(high, 0x1b3) + Math.imul(previousLow, 0x100) + carry) >>> 0;
            }
            const hash = high.toString(16).padStart(8, "0") + low.toString(16).padStart(8, "0");
            return [bytes.length, hash];
        };
        const canonical = program && ts.createGetCanonicalFileName(program.useCaseSensitiveFileNames());
        const occurrences = new Map<string, number>();
        const materialUnits = units.filter(unit => unit.content !== undefined);
        const unitPaths = materialUnits.map(unit => program && ts.toPath(unit.name, program.getCurrentDirectory(), canonical!));
        const canonicalUnitIndex = new Map<string, number>();
        for (let index = 0; index < unitPaths.length; index++) {
            const unitPath = unitPaths[index];
            if (unitPath) canonicalUnitIndex.set(unitPath, index);
        }
        const exactPaths = new Set<string>();
        const records = materialUnits.map((unit, unitIndex) => {
            const occurrence = occurrences.get(unit.name) ?? 0;
            occurrences.set(unit.name, occurrence + 1);
            const unitPath = unitPaths[unitIndex];
            const sourceFile = unitPath && program!.getSourceFileByPath(unitPath);
            const byName = program?.getSourceFile(unit.name);
            if (sourceFile !== byName) {
                throw new Error(\`SourceFile lookup disagrees for \${unit.name}\`);
            }

            let status = "a";
            let redirect;
            if (unitPath && canonicalUnitIndex.get(unitPath) !== unitIndex) {
                status = "r";
                redirect = ["s"];
            }
            else if (sourceFile?.redirectInfo) {
                status = "r";
                redirect = ["p", sourceFile.redirectInfo.redirectTarget.path];
            }
            else if (sourceFile && sourceFile.path !== unitPath) {
                status = "r";
                redirect = ["i", sourceFile.path];
            }
            else if (sourceFile && sourceFile.text !== unit.content) {
                status = "r";
                redirect = ["i", sourceFile.path];
            }
            else if (sourceFile) {
                status = "e";
                if (exactPaths.has(sourceFile.path)) {
                    throw new Error(\`duplicate exact SourceFile identity \${sourceFile.path}\`);
                }
                exactPaths.add(sourceFile.path);
            }

            const [contentByteLength, contentHash] = status === "e"
                ? contentIdentity(sourceFile!.text)
                : [undefined, undefined];

            const diagnosticCodes = sourceFile ? diagnostics
                .filter(diagnostic => diagnostic.category === ts.DiagnosticCategory.Error && diagnostic.file === sourceFile)
                .map(diagnostic => diagnostic.code)
                .sort((left, right) => left - right)
                .filter((code, index, all) => index === 0 || code !== all[index - 1]) : [];
            const checkJs = sourceFile
                ? ts.isCheckJsEnabledForFile(sourceFile, options) === true
                : undefined;
            const isTypeScript = !!sourceFile && (sourceFile.scriptKind === ts.ScriptKind.TS || sourceFile.scriptKind === ts.ScriptKind.TSX);
            const skipped = !!sourceFile && !!program && ts.skipTypeChecking(sourceFile, options, program);
            const externalModuleIndicator = !sourceFile ? undefined
                : !sourceFile.externalModuleIndicator ? "n"
                : sourceFile.externalModuleIndicator === true ? "f"
                : "s";
            const commonJsModuleIndicator = sourceFile
                ? !!sourceFile.commonJsModuleIndicator
                : undefined;
            const impliedNodeFormat = !sourceFile ? undefined
                : sourceFile.impliedNodeFormat === ts.ModuleKind.CommonJS ? "c"
                : sourceFile?.impliedNodeFormat === ts.ModuleKind.ESNext ? "m"
                : "n";
            const sourceType = !sourceFile ? undefined
                : externalModuleIndicator === "s" || impliedNodeFormat === "m" ? "m"
                : commonJsModuleIndicator || impliedNodeFormat === "c" ? "c"
                : externalModuleIndicator === "f" ? "m"
                : "s";

            return {
                name: unit.name,
                occurrence,
                status,
                contentByteLength,
                contentHash,
                diagnosticCodes,
                checkJs,
                semanticChecked: status === "e" && !skipped && (isTypeScript || checkJs),
                externalModuleIndicator,
                commonJsModuleIndicator,
                impliedNodeFormat,
                sourceType,
                redirect,
            };
        });
        return {
            name: this.configuredName,
            globalDiagnosticCodes: diagnostics
                .filter(diagnostic => diagnostic.category === ts.DiagnosticCategory.Error && !diagnostic.file)
                .map(diagnostic => diagnostic.code)
                .sort((left, right) => left - right)
                .filter((code, index, all) => index === 0 || code !== all[index - 1]),
            units: records,
        };
    }
`;
    const exportHelper = `

export function collectCompilerManifest(fileName: string) {
    const test = CompilerTest.getConfigurations(fileName);
    const configurations = test.configurations?.length ? test.configurations : [undefined];
    return configurations.map(configuration => {
        const payload = TestCaseParser.makeUnitsFromTest(test.content!, test.file);
        const compilerTest = new CompilerTest(fileName, payload, configuration);
        return { configuration: configuration ?? {}, ...compilerTest.manifest(payload.testUnitData) };
    });
}
`;

    await build({
        entryPoints: [path.join(scriptRoot, "entry.ts")],
        outfile: path.join(tempRoot, "generator.cjs"),
        bundle: true,
        platform: "node",
        target: "node20",
        format: "cjs",
        treeShaking: true,
        logLevel: "warning",
        plugins: [{
            name: "typescript-manifest-probe",
            setup(build) {
                build.onResolve({ filter: /^__TS_ROOT__\// }, args => ({
                    path: path.join(fixtureRoot, args.path.slice("__TS_ROOT__/".length)),
                }));
                build.onResolve({ filter: /diagnosticInformationMap\.generated\.js$/ }, () => ({
                    path: "diagnostic-information",
                    namespace: "generated",
                }));
                build.onResolve({ filter: /^(chai|diff|source-map-support)$/ }, args => ({
                    path: args.path,
                    namespace: "stub",
                }));
                build.onLoad({ filter: /.*/, namespace: "generated" }, () => ({
                    contents: diagnosticModule(),
                    loader: "ts",
                    resolveDir: path.join(fixtureRoot, "src/compiler"),
                }));
                build.onLoad({ filter: /.*/, namespace: "stub" }, args => {
                    if (args.path === "chai") {
                        return {
                            contents: "import nodeAssert from 'node:assert'; const base = value => nodeAssert.ok(value); base.equal = nodeAssert.equal; base.deepEqual = nodeAssert.deepEqual; base.isTrue = (value, message) => nodeAssert.strictEqual(value, true, message); base.isFalse = (value, message) => nodeAssert.strictEqual(value, false, message); export const assert = base; export const expect = () => ({});",
                            loader: "js",
                        };
                    }
                    return {
                        contents: "export default {}; export const createTwoFilesPatch = () => '';",
                        loader: "js",
                    };
                });
                build.onLoad({ filter: /compilerRunner\.ts$/ }, args => {
                    if (args.path !== compilerRunner) return;
                    let contents = fs.readFileSync(args.path, "utf8");
                    if (!contents.includes(injectMarker) || !contents.includes(methodMarker)) {
                        throw new Error("TypeScript compiler runner markers changed");
                    }
                    contents = contents.replace(methodMarker, helper + "\n" + methodMarker) + exportHelper;
                    return { contents, loader: "ts", resolveDir: path.dirname(args.path) };
                });
                build.onLoad({ filter: /harnessIO\.ts$/ }, args => {
                    if (args.path !== harnessIo) return;
                    let contents = fs.readFileSync(args.path, "utf8");
                    const marker = "    const workspaceRoot = Utils.findUpRoot();";
                    if (!contents.includes(marker)) {
                        throw new Error("TypeScript harness IO marker changed");
                    }
                    contents = contents.replace(
                        marker,
                        "    const workspaceRoot = process.env.TS_MANIFEST_TEMP_ROOT!;",
                    );
                    return { contents, loader: "ts", resolveDir: path.dirname(args.path) };
                });
                build.onLoad({ filter: /_namespaces\/Harness\.ts$/ }, args => {
                    if (args.path !== testRunnerHarness) return;
                    return {
                        contents: "export * from '../../harness/_namespaces/Harness.js';",
                        loader: "ts",
                        resolveDir: path.dirname(args.path),
                    };
                });
            },
        }],
    });
}
