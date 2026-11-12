import assert from "node:assert/strict";
import test, { describe } from "node:test";

import { parseSync, transformSync as coreTransformSync } from "../src-js/index.js";
import * as standalone from "../../transformer/src-js/index.js";

test("@swc-next/transformer exposes only transformSync", () => {
  assert.deepEqual(Object.keys(standalone), ["transformSync"]);
});

for (const { name, transformSync } of [
  { name: "@swc-next/core", transformSync: coreTransformSync },
  { name: "@swc-next/transformer", transformSync: standalone.transformSync },
]) {
  describe(name, () => {
    function syntaxError(source, options) {
      let captured;
      assert.throws(() => transformSync(source, options), error => {
        assert.ok(error instanceof SyntaxError);
        captured = error;
        return true;
      });
      return captured;
    }

    test("transformSync returns code and diagnostics synchronously", () => {
      const result = transformSync("const value = 1;");
      assert.deepEqual(result, { code: "const value = 1;\n", diagnostics: [] });
      assert.deepEqual(transformSync("const value = 1;", null), result);
      assert.deepEqual(transformSync(""), { code: "", diagnostics: [] });
    });

    test("transformSync infers language and source type from filename", () => {
      const source = "const value: number = 1;";
      assert.equal(transformSync(source, { filename: "input.ts" }).code, "const value = 1;\n");

      assert.equal(transformSync(source, { filename: "input.mts" }).code, "const value = 1;\nexport {};\n");

      assert.equal(transformSync(source, { filename: "input.js", parser: { lang: "ts" } }).code, "const value = 1;\n");

      assert.equal(transformSync("return 1;", { filename: "input.cts" }).code, "return 1;\n");

      syntaxError("return 1;", { filename: "input.mjs" });

      assert.equal(transformSync("return 1;", {
        filename: "input.mjs",
        parser: { sourceType: "commonjs" },
      }).code, "return 1;\n");

      syntaxError("return 1;", {
        filename: "input.cjs",
        parser: { sourceType: "module" },
      });
    });

    test("transformSync makes semantic diagnostics optional", () => {
      const source = "let value; let value;";
      const disabled = { parser: { semanticErrors: false } };
      const enabled = { parser: { semanticErrors: true } };
      const result = transformSync(source, disabled);
      assert.deepEqual(result, { code: "let value;\nlet value;\n", diagnostics: [] });
      assert.deepEqual(transformSync(source), result);
      assert.ok(syntaxError(source, enabled).diagnostics.length > 0);

      syntaxError("const = 1;", disabled);
      syntaxError("const view = <svg:path />;", { ...disabled, filename: "input.jsx" });
    });

    test("transformSync forwards TypeScript defaults and disabling", () => {
      const source = "const value: number = 1;";
      const options = { filename: "input.ts" };
      const defaults = transformSync(source, options);
      assert.deepEqual(transformSync(source, { ...options, transformer: { typescript: true } }), defaults);
      assert.deepEqual(transformSync(source, { ...options, transformer: { typescript: {} } }), defaults);

      const preserved = transformSync(source, { ...options, transformer: { typescript: false } });
      assert.equal(preserved.code, "const value: number = 1;\n");
    });

    test("transformSync forwards TypeScript emission options", () => {
      const source = "const enum E { X = 1 } function read(E: { X: number }) { return E.X; } export const value = E.X;";
      const defaults = transformSync(source, { parser: { lang: "ts" } });
      assert.deepEqual(transformSync(source, {
        parser: { lang: "ts", semanticErrors: false },
      }), defaults);
      assert.deepEqual(transformSync(source, {
        parser: { lang: "ts", semanticErrors: true },
      }), defaults);
      assert.ok(!defaults.code.includes("var E"));
      assert.ok(defaults.code.includes("export const value = 1"));
      assert.ok(defaults.code.includes("return E.X"));

      const preserved = transformSync(source, {
        parser: { lang: "ts" },
        transformer: { typescript: { preserveConstEnums: true } },
      });
      assert.ok(preserved.code.includes("var E"));
      assert.ok(preserved.code.includes("export const value = 1"));

      const verbatim = transformSync(source, {
        parser: { lang: "ts" },
        transformer: { typescript: { verbatimModuleSyntax: true } },
      });
      assert.ok(verbatim.code.includes("var E"));
      assert.ok(verbatim.code.includes("export const value = E.X"));
    });

    test("transformSync forwards JSX defaults, runtime options, and disabling", () => {
      const source = "export const view: unknown = <section />;";
      const options = { filename: "input.tsx" };
      const defaults = transformSync(source, options);
      assert.ok(defaults.code.includes('"react/jsx-runtime"'));
      assert.ok(!defaults.code.includes(": unknown"));
      assert.deepEqual(transformSync(source, { ...options, transformer: { jsx: true } }), defaults);
      assert.deepEqual(transformSync(source, { ...options, transformer: { jsx: {} } }), defaults);

      const preserved = transformSync(source, { ...options, transformer: { jsx: false } });
      assert.ok(preserved.code.includes("<section />"));
      assert.ok(!preserved.code.includes(": unknown"));

      const automatic = transformSync(source, {
        ...options,
        transformer: { jsx: { importSource: "preact", pure: false } },
      });
      assert.ok(automatic.code.includes('"preact/jsx-runtime"'));
      assert.ok(!automatic.code.includes("#__PURE__"));

      const classic = transformSync("export const view = <><section /></>;", {
        filename: "input.jsx",
        transformer: { jsx: { runtime: "classic", pragma: "h", pragmaFrag: "Fragment" } },
      });
      assert.ok(classic.code.includes("h(Fragment"));
    });

    test("transformSync forwards JSX development filename and Refresh options", () => {
      const source = "export function App() { return <section />; }";
      const options = { filename: "src/view.jsx" };
      const jsx = { development: true, sourceSelf: true };
      const development = transformSync(source, { ...options, transformer: { jsx } });
      assert.ok(development.code.includes("src/view.jsx"));
      assert.ok(development.code.includes("jsxDEV"));

      const refresh = transformSync(source, { ...options, transformer: { jsx: { ...jsx, refresh: true } } });
      assert.ok(refresh.code.includes("$RefreshReg$"));
      assert.deepEqual(transformSync(source, {
        ...options,
        transformer: { jsx: { ...jsx, refresh: {} } },
      }), refresh);

      assert.deepEqual(transformSync(source, {
        ...options,
        transformer: { jsx: { ...jsx, refresh: false } },
      }), development);
    });

    test("transformSync forwards feature overrides", () => {
      const source = "class C { static { run(); } }";
      assert.ok(transformSync(source).code.includes("static {"));
      assert.ok(!transformSync(source, {
        transformer: { features: { "class-static-block": true } },
      }).code.includes("static {"));

      const catchSource = "try { run(); } catch { recover(); }";
      const defaults = transformSync(catchSource);
      const lowered = transformSync(catchSource, {
        transformer: { features: { "optional-catch-binding": true } },
      });
      assert.match(lowered.code, /catch \(\w+\)/);
      assert.deepEqual(transformSync(catchSource, {
        transformer: { features: { "optional-catch-binding": false } },
      }), defaults);
    });

    test("transformSync keeps parser comment collection separate from output comments", () => {
      const source = "/* keep me */ const value = 1;";
      assert.ok(transformSync(source).code.includes("keep me"));
      assert.equal(transformSync(source, { comments: "none" }).code, "const value = 1;\n");
      assert.equal(transformSync(source, { comments: "all", parser: { comments: "none" } }).code, "const value = 1;\n");
      assert.equal(transformSync(source, { comments: "none", parser: { comments: "all" } }).code, "const value = 1;\n");

      const pragma = transformSync("/** @jsxRuntime classic */ export const view = <section />;", {
        filename: "input.jsx",
        comments: "none",
        transformer: { jsx: { pure: false } },
      });
      assert.ok(pragma.code.includes("React.createElement"));
      assert.ok(!pragma.code.includes("@jsxRuntime"));
    });

    test("transformSync forwards codegen options", () => {
      const result = transformSync('const value = "😀</script>";', {
        generator: { asciiOnly: true, minify: true, inlineScript: true },
      });
      assert.equal(result.code, 'const value="\\u{1F600}<\\/script>";');
      assert.equal(transformSync('const value = "😀";', {
        generator: { asciiOnly: false, minify: false },
      }).code, 'const value = "😀";\n');
    });

    test("transformSync preserves strings across the source and result boundaries", () => {
      const source = 'const 文 = "é😀\\0\\uD800";';
      const result = transformSync(source);
      const { program } = parseSync(result.code);
      assert.equal(program.body[0].declarations[0].id.name, "文");
      assert.equal(program.body[0].declarations[0].init.value, "é😀\0\uD800");

      assert.ok(transformSync('#!/usr/bin/env node\n"use strict"; run();').code.startsWith('#!/usr/bin/env node\n"use strict";'));
      assert.equal(transformSync("const second = 2;").code, "const second = 2;\n");
    });

    test("transformSync returns UTF-16 diagnostics with related labels and help", () => {
      const syntaxSource = "/* 😀 */ const = 1;";
      const syntax = syntaxError(syntaxSource, { filename: "broken.js" });
      assert.match(syntax.message, /broken\.js/);
      assert.equal(syntax.diagnostics[0].severity, "error");
      assert.equal(syntax.diagnostics[0].start, syntaxSource.indexOf("="));
      assert.equal(syntax.diagnostics[0].end, syntaxSource.indexOf("=") + 1);

      const semanticSource = "/* 😀 */ let 文; let 文;";
      const semantic = syntaxError(semanticSource, { parser: { semanticErrors: true } });
      assert.equal(semantic.diagnostics[0].start, semanticSource.lastIndexOf("文"));
      assert.ok(semantic.diagnostics[0].labels.some(label => label.start === semanticSource.indexOf("文")));

      const jsxSource = "/* 😀 */ <开始></结束>";
      const jsx = syntaxError(jsxSource, { filename: "input.jsx" });
      assert.ok(jsx.diagnostics[0].help.length > 0);
      assert.equal(jsx.diagnostics[0].start, jsxSource.indexOf("结束"));
      assert.equal(jsx.diagnostics[0].end, jsxSource.indexOf("结束") + 2);
      assert.equal(jsx.diagnostics[0].labels[0].start, jsxSource.indexOf("开始"));
    });

    test("transformSync returns diagnostics from the Rust transform", () => {
      const source = "export const view = <svg:path />;";
      const error = syntaxError(source, { filename: "input.jsx" });
      assert.ok(error.diagnostics.some(diagnostic => diagnostic.severity === "error"));

      const result = transformSync(source, {
        filename: "input.jsx",
        transformer: { jsx: { throwIfNamespace: false } },
      });
      assert.ok(result.code.includes('"svg:path"'));
      assert.deepEqual(result.diagnostics, []);
    });

    test("transformSync returns source maps from Rust", () => {
      const source = "export const value: number = 1;";
      const options = { filename: "src/你好.ts", sourceMap: true };
      const result = transformSync(source, options);
      const map = JSON.parse(result.map);
      assert.equal(map.version, 3);
      assert.deepEqual(map.sources, [options.filename]);
      assert.deepEqual(map.sourcesContent, [source]);
      assert.ok(map.mappings.length > 0);
      assert.equal(transformSync(source, { ...options, sourceMap: false }).map, undefined);
    });

    test("transformSync rejects minification and only compacts with codegen", () => {
      const source = "export const value = 1 + 2;";
      assert.throws(() => transformSync(source, { minify: true }), TypeError);
      assert.throws(() => transformSync(source, { minify: false }), TypeError);
      assert.throws(() => transformSync(source, { minify: {} }), TypeError);
      assert.throws(() => transformSync(source, { parse: {} }), TypeError);
      assert.equal(transformSync(source, { generator: { minify: true } }).code, "export const value=1+2;");
      assert.equal(transformSync(source).code, "export const value = 1 + 2;\n");
    });

    test("transformSync rejects malformed and unsupported options", () => {
      const source = "const value = 1;";
      assert.throws(() => transformSync(null), TypeError);
      assert.throws(() => transformSync(source, []), TypeError);
      assert.throws(() => transformSync(source, false), TypeError);
      assert.throws(() => transformSync(source, { filename: 1 }), TypeError);
      assert.throws(() => transformSync(source, { parser: { lang: "css" } }), TypeError);
      assert.throws(() => transformSync(source, { parser: "invalid" }), TypeError);
      assert.throws(() => transformSync(source, { parser: { semanticErrors: "false" } }), TypeError);
      assert.throws(() => transformSync(source, { parser: { sourceType: "invalid" } }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { typescript: 1 } }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { typescript: { unsupported: true } } }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { jsx: { runtime: "invalid" } } }), TypeError);
      assert.throws(() => transformSync(source, { generator: { minify: "true" } }), TypeError);

      assert.throws(() => transformSync(source, { js: {} }), TypeError);
      assert.throws(() => transformSync(source, { plugins: [] }), TypeError);
      assert.throws(() => transformSync(source, { typescript: {} }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { proposal: { unknown: true } } }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { typescript: { jsx: {} } } }), TypeError);
      assert.throws(() => transformSync(source, { target: "es2015" }), TypeError);
      assert.throws(() => transformSync(source, { lang: "ts" }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { decorators: false } }), TypeError);
      assert.throws(() => transformSync(source, { transformer: { assumptions: { unknown: true } } }), TypeError);
      assert.throws(() => transformSync(source, { sourceMap: "inline" }), TypeError);
      assert.throws(() => transformSync(source, { minify: { compress: { unknown: true } } }), TypeError);
      assert.throws(() => transformSync(source, { comments: "some" }), TypeError);
      assert.throws(() => transformSync(source, { errorRecovery: true }), TypeError);
    });
  });
}
