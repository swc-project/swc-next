import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { registerHooks } from "node:module";
import test, { describe } from "node:test";

import { parseSync, transformSync as coreTransformSync } from "../src-js/index.js";
import * as standalone from "../../transformer/src-js/index.js";

for (const { name, entrypoint, transformSync } of [
  { name: "@swc-next/core", entrypoint: "../src-js/index.js", transformSync: coreTransformSync },
  { name: "@swc-next/transformer", entrypoint: "../../transformer/src-js/index.js", transformSync: standalone.transformSync },
]) {
  describe(`${name} plugins`, () => {
    const beforePlugin = { name: "noop-before", visitor: {} };
    const afterPlugin = { name: "noop-after", stage: "beforeCodegen", visitor: {} };
    const pluginConfigurations = [[], [beforePlugin], [afterPlugin], [beforePlugin, afterPlugin]];

    function assertSameTransform(source, options) {
      const expected = transformSync(source, options);
      for (const plugin of pluginConfigurations) {
        assert.deepEqual(transformSync(source, { ...options, plugins: plugin }), expected);
      }
      return expected;
    }

    function assertSameTransformError(source, options) {
      function capture(options) {
        let captured;
        assert.throws(() => transformSync(source, options), (error) => {
          captured = error;
          return true;
        });
        return captured;
      }

      const expected = capture(options);
      for (const plugin of pluginConfigurations) {
        const actual = capture({ ...options, plugins: plugin });
        assert.equal(actual.constructor, expected.constructor);
        assert.equal(actual.message, expected.message);
        assert.deepEqual(actual.diagnostics, expected.diagnostics);
      }
      return expected;
    }

    test("transformSync transfers the AST only for configured plugin stages", async () => {
      const namespace = `swc-test:transform-${randomUUID()}`;
      const subject = new URL(`${entrypoint}?${randomUUID()}`, import.meta.url).href;
      const countsUrl = `${namespace}/counts`;
      const modules = new Map([[countsUrl, "export const counts = { encode: 0, decode: 0 };"]]);
      const wrappers = new Map();
      for (const [name, method] of [["encoder", "encode"], ["decoder", "decode"]]) {
        const specifier = `@swc-next/${name}`;
        const original = JSON.stringify(import.meta.resolve(specifier));
        const wrapper = `${namespace}/${name}`;
        wrappers.set(specifier, wrapper);
        modules.set(wrapper, `
          export * from ${original};
          import { ${method} as run } from ${original};
          import { counts } from ${JSON.stringify(countsUrl)};
          export function ${method}(...args) {
            counts.${method}++;
            return run(...args);
          }
        `);
      }
      const hooks = registerHooks({
        resolve(specifier, context, nextResolve) {
          const url = context.parentURL === subject ? wrappers.get(specifier) : undefined;
          if (url || modules.has(specifier)) return { url: url ?? specifier, shortCircuit: true };
          return nextResolve(specifier, context);
        },
        load(url, context, nextLoad) {
          if (modules.has(url)) return { format: "module", source: modules.get(url), shortCircuit: true };
          return nextLoad(url, context);
        },
      });
      let instrumented;
      let counts;
      try {
        instrumented = (await import(subject)).transformSync;
        counts = (await import(countsUrl)).counts;
      } finally {
        hooks.deregister();
      }

      function check(plugin, expectedTransfers) {
        counts.encode = counts.decode = 0;
        const result = instrumented("const value: number = 1;", { filename: "input.ts", plugins: plugin });
        assert.deepEqual(result, { code: "const value = 1;\n", diagnostics: [] });
        assert.deepEqual(counts, { encode: expectedTransfers, decode: expectedTransfers });
      }

      check(undefined, 0);
      check([], 0);
      check([beforePlugin], 1);
      check([afterPlugin], 1);
      check([beforePlugin, afterPlugin], 2);
    });

    test("transformSync keeps plugin stage configurations consistent across source encodings", () => {
      assertSameTransform("");

      const result = assertSameTransform('const 文 = "é😀\\0\\uD800";');
      assert.equal(parseSync(result.code).program.body[0].declarations[0].init.value, "é😀\0\uD800");

      const directives = assertSameTransform('#!/usr/bin/env node\n"use strict"; /* comment */ run();', { comments: "none" });
      assert.ok(directives.code.startsWith('#!/usr/bin/env node\n"use strict";'));
      assert.ok(!directives.code.includes("comment"));
    });

    test("transformSync preserves numeric literal spellings across plugin boundaries", () => {
      assert.equal(assertSameTransform("const value = 0;").code, "const value = 0;\n");

      assert.equal(assertSameTransform("const value = -0;").code, "const value = -0;\n");

      assert.equal(assertSameTransform("const value = 0x0;").code, "const value = 0x0;\n");

      assert.equal(assertSameTransform("const value = 0n;").code, "const value = 0n;\n");

      assert.equal(assertSameTransform("const value = 9007199254740993n;").code, "const value = 9007199254740993n;\n");

      assert.equal(assertSameTransform("const value = 0x20000000000001n;").code, "const value = 0x20000000000001n;\n");
    });

    test("transformSync forwards options consistently across plugin stage configurations", () => {
      const typescript = { filename: "input.ts" };
      const retainedTypes = {
        filename: "input.js",
        transformer: { typescript: false },
        parser: { lang: "ts", preserveParens: false },
      };
      const commonjs = { filename: "input.cjs", parser: { sourceType: "commonjs" } };
      const moduleOverride = { filename: "input.cjs", parser: { sourceType: "module" } };
      const staticBlocks = { transformer: { features: { "class-static-block": true } } };
      const escapedCode = {
        generator: { asciiOnly: true, minify: true, inlineScript: true },
      };

      assert.equal(assertSameTransform("const value: number = 1;", typescript).code, "const value = 1;\n");

      assert.equal(assertSameTransform("const value: number = (1);", retainedTypes).code, "const value: number = 1;\n");

      assertSameTransform("return 1;", commonjs);

      assertSameTransform("export const value = 1;", moduleOverride);

      assertSameTransform("class C { static { run(); } }", staticBlocks);

      assertSameTransform('const value = "😀</script>";', escapedCode);
    });

    test("transformSync lowers JSX by default and supports preserving it", () => {
      const source = 'export const view: unknown = <section>文😀 &amp; text</section>;';
      const options = { filename: "input.tsx", comments: "none" };
      assertSameTransform(source, {
        ...options,
        transformer: { jsx: { pure: false } },
      });

      const preserved = assertSameTransform(source, {
        ...options,
        transformer: { jsx: false },
      });
      assert.ok(preserved.code.includes("<section>文😀 &amp; text</section>"));
      assert.ok(!preserved.code.includes("react/jsx-runtime"));
      assert.ok(!preserved.code.includes(": unknown"));
    });

    test("transformSync preserves disabled Rust syntax through plugin edits", () => {
      const source = "const view: unknown = <section />;";
      const options = {
        filename: "input.tsx",
        transformer: {
          typescript: false,
          jsx: false,
          features: { "class-static-block": false, "optional-catch-binding": false },
        },
        parser: { semanticErrors: true },
      };
      const expected = assertSameTransform(source, options);
      assert.ok(expected.code.includes(": unknown"));
      assert.ok(expected.code.includes("<section />"));

      for (const stage of ["afterParse", "beforeCodegen"]) {
        const result = transformSync(source, {
          ...options,
          plugins: [{
            name: "rename",
            stage,
            visitor: {
              Identifier(node) {
                if (node.name === "view") node.name = "edited";
              },
            },
          }],
        });
        assert.deepEqual(result, { ...expected, code: expected.code.replace("view", "edited") });
      }
    });

    test("transformSync preserves comments in Rust and rejects lossy plugin boundaries", () => {
      const source = "/* keep me */ const value = 1;";
      assert.ok(!assertSameTransform(source, { comments: "none" }).code.includes("keep me"));
      assert.ok(!assertSameTransform(source, { comments: "none", parser: { comments: "all" } }).code.includes("keep me"));
      assert.ok(!assertSameTransform(source, { parser: { comments: "none" } }).code.includes("keep me"));

      for (const plugins of [[beforePlugin], [afterPlugin], [beforePlugin, afterPlugin]]) {
        let visited = false;
        const observing = plugins.map(plugin => ({
          ...plugin,
          visitor: { Program() { visited = true; } },
        }));
        assert.throws(() => transformSync(source, { plugins: observing }), error => {
          assert.ok(error instanceof TypeError);
          assert.match(error.message, /comments/);
          return true;
        });
        assert.equal(visited, false);
      }
      assertSameTransform("const value = 1;");
    });

    test("transformSync preserves diagnostic metadata across plugin stage configurations", () => {
      const syntaxSource = "/* 😀 */ const = 1;";
      const syntax = assertSameTransformError(syntaxSource, {
        filename: "broken.js", comments: "none", parser: { semanticErrors: false },
      });
      assert.ok(syntax instanceof SyntaxError);
      assert.equal(syntax.diagnostics[0].start, syntaxSource.indexOf("="));
      assert.equal(syntax.diagnostics[0].end, syntaxSource.indexOf("=") + 1);

      const semanticSource = "/* 😀 */ let 文; let 文;";
      const semantic = assertSameTransformError(semanticSource, {
        comments: "none", parser: { semanticErrors: true },
      });
      assert.equal(semantic.diagnostics[0].start, semanticSource.lastIndexOf("文"));
      assert.ok(semantic.diagnostics[0].labels.some((label) => label.start === semanticSource.indexOf("文")));

      const jsxSource = "/* 😀 */ <开始></结束>";
      const jsx = assertSameTransformError(jsxSource, { filename: "input.jsx", comments: "none" });
      assert.ok(jsx.diagnostics[0].help.length > 0);
      assert.equal(jsx.diagnostics[0].start, jsxSource.indexOf("结束"));
      assert.equal(jsx.diagnostics[0].end, jsxSource.indexOf("结束") + 2);
      assert.equal(jsx.diagnostics[0].labels[0].start, jsxSource.indexOf("开始"));
      assert.equal(jsx.diagnostics[0].labels[0].end, jsxSource.indexOf("开始") + 2);
    });

    test("transformSync forwards semantic checks across plugin stage configurations", () => {
      const source = "let value; let value;";
      const disabled = { parser: { semanticErrors: false } };
      const enabled = { parser: { semanticErrors: true } };
      const result = assertSameTransform(source, disabled);
      assert.deepEqual(result.diagnostics, []);
      assert.deepEqual(assertSameTransform(source), result);
      assert.ok(assertSameTransformError(source, enabled) instanceof SyntaxError);
    });

    test("transformSync validates options consistently across plugin stage configurations", () => {
      const source = "const value = 1;";
      assertSameTransformError(null);

      assertSameTransformError(source, { filename: 1 });

      assertSameTransformError(source, { parser: "invalid" });

      assertSameTransformError(source, { transformer: { typescript: "false" } });

      assertSameTransformError(source, { generator: { minify: "true" } });

      assertSameTransformError(source, { transformer: { typescript: { jsx: { automatic: "true" } } } });

      assertSameTransformError(source, { transformer: { features: { "class-static-block": "true" } } });

      assertSameTransformError(source, { proposal: { preventExtensions: "true" } });
    });

    test("transformSync rejects unsupported options before invoking plugins", () => {
      function invalid(options, field) {
        let visited = false;
        for (const stage of ["afterParse", "beforeCodegen"]) {
          assert.throws(() => transformSync("const value = 1;", {
            ...options,
            plugins: [{
              name: "must-not-run",
              stage,
              visitor: { Program() { visited = true; } },
            }],
          }), (error) => {
            assert.ok(error instanceof TypeError);
            if (field !== undefined) assert.ok(error.message.includes(field));
            return true;
          });
        }
        assert.equal(visited, false);
      }

      invalid({ minify: true }, "minify");
      invalid({ minify: false }, "minify");
      invalid({ minify: {} }, "minify");
      invalid({ parse: {} }, "parse");
      invalid({ js: { transformer: {} } }, "js");
      invalid({ typescript: {} }, "typescript");
      invalid({ es2022: {} }, "es2022");
      invalid({ proposal: {} }, "proposal");
      invalid({ sourceMaps: true }, "sourceMaps");
      invalid({ isModule: true }, "isModule");
      invalid({ jsc: {} }, "jsc");
      invalid({ swcNext: {} }, "swcNext");
      invalid({ lang: "css" }, "lang");
      invalid({ target: "es2015" }, "target");
      invalid({ sourceMap: true }, "sourceMap");
      invalid({ errorRecovery: true }, "errorRecovery");
      invalid({ comments: "some" }, "some");
      invalid({ parser: { syntax: "typescript" } }, "syntax");
      invalid({ parser: { lang: "css" } }, "css");
      invalid({ parser: { comments: "flat" } }, "flat");
      invalid({ parser: { semanticErrors: "false" } });
      invalid({ parser: { errorRecovery: false } }, "errorRecovery");
      invalid({ generator: { charset: "ascii" } }, "charset");
      invalid({ generator: { preamble: "banner" } }, "preamble");
      invalid({ transformer: { typescript: { unsupported: true } } });
      invalid({ transformer: { typescript: { jsx: {} } } });
      invalid({ transformer: { typescript: "true" } });
      invalid({ transformer: { features: { unsupported: true } } }, "unsupported");
      invalid({ transformer: { jsx: { useSpread: true } } });
      invalid({ transformer: { module: { type: "commonjs" } } }, "module");
      invalid({ transformer: { helpers: "external" } }, "helpers");
      invalid({ transformer: { assumptions: { unsupported: true } } }, "unsupported");
      invalid({ transformer: { define: {} } }, "define");
      invalid({ transformer: { proposal: { unsupported: true } } }, "unsupported");
    });

    test("transformSync exposes the complete program and supports statement edits", () => {
      const imports = [];
      const source = 'import value from "dependency"; debugger; if (value) debugger;';
      const result = transformSync(source, {
        filename: "input.js",
        plugins: [{
          name: "remove-debugger",
          visitor: {
            Program(program) {
              imports.push(...program.body
                .filter((node) => node.type === "ImportDeclaration")
                .map((node) => node.source.value));
              assert.equal(program.body.length, 3);
            },
            DebuggerStatement(_, ctx) {
              if (ctx.index !== null) ctx.remove();
              else ctx.replace({ type: "EmptyStatement" });
            },
          },
        }],
      });

      assert.deepEqual(imports, ["dependency"]);
      const body = parseSync(result.code).program.body;
      assert.equal(body.length, 2);
      assert.equal(body[0].type, "ImportDeclaration");
      assert.equal(body[1].type, "IfStatement");
      assert.equal(body[1].consequent.type, "EmptyStatement");
      assert.deepEqual(result.diagnostics, []);
    });

    test("transformSync runs plugins on either side of the Rust transform", () => {
      let beforeProgram;
      let afterProgram;
      const seen = [];
      const result = transformSync("const value: number = 1;", {
        filename: "input.ts",
        plugins: [{
          name: "afterParse",
          visitor: {
            Program(program) { beforeProgram = program; },
            Identifier(node) {
              assert.equal(node.typeAnnotation.type, "TSTypeAnnotation");
              node.name = "beforeValue";
            },
            TSNumberKeyword() { seen.push("type"); },
          },
        }, {
          name: "beforeCodegen",
          stage: "beforeCodegen",
          visitor: {
            Program(program) { afterProgram = program; },
            Identifier(node) {
              assert.equal(node.name, "beforeValue");
              assert.ok(node.typeAnnotation == null);
              node.name = "afterValue";
            },
            TSNumberKeyword() { assert.fail("TypeScript was not removed"); },
          },
        }],
      });

      assert.deepEqual(seen, ["type"]);
      assert.notEqual(beforeProgram, afterProgram);
      assert.equal(result.code, "const afterValue = 1;\n");
    });

    test("transformSync gives single-stage plugins the correct AST and prints their edits", () => {
      const source = "const value: number = 1;";
      for (const stage of ["afterParse", "beforeCodegen"]) {
        let visited = false;
        const result = transformSync(source, {
          filename: "input.ts",
          plugins: [{
            name: "rename",
            stage,
            visitor: {
              Identifier(node) {
                visited = true;
                if (stage === "afterParse") assert.equal(node.typeAnnotation.type, "TSTypeAnnotation");
                else assert.ok(node.typeAnnotation == null);
                node.name = "renamed";
              },
            },
          }],
        });

        assert.equal(visited, true);
        assert.deepEqual(result, { code: "const renamed = 1;\n", diagnostics: [] });
      }
    });

    test("transformSync runs a complete walk per plugin in stage order", () => {
      const seen = [];
      function rename(name, stage, from, to) {
        return {
          name,
          stage,
          visitor: {
            Program() { seen.push(`${name}:program`); },
            Identifier(node) {
              seen.push(`${name}:${node.name}`);
              assert.equal(node.name, from);
              node.name = to;
            },
          },
        };
      }

      const result = transformSync("const original = 1;", {
        plugins: [
          rename("after-first", "beforeCodegen", "second", "third"),
          rename("before-first", "afterParse", "original", "first"),
          rename("after-second", "beforeCodegen", "third", "finalValue"),
          rename("before-second", "afterParse", "first", "second"),
        ],
      });

      assert.deepEqual(seen, [
        "before-first:program", "before-first:original",
        "before-second:program", "before-second:first",
        "after-first:program", "after-first:second",
        "after-second:program", "after-second:third",
      ]);
      assert.equal(result.code, "const finalValue = 1;\n");
    });

    test("transformSync infers syntax from filename and forwards Rust parser options", () => {
      assert.ok(assertSameTransformError("const value: number = 1;", {
        filename: "input.ts",
        parser: { lang: "js" },
      }) instanceof SyntaxError);

      assertSameTransform("const view = <section />;", {
        parser: { lang: "jsx" },
        transformer: { jsx: { pure: false } },
      });

      assertSameTransform("const view: unknown = <section />;", {
        parser: { lang: "tsx" },
        transformer: { jsx: { pure: false } },
      });

      assertSameTransform("export const value: number;", {
        filename: "input.d.ts",
        parser: { lang: "dts" },
      });

      function sourceType(options) {
        let actual;
        transformSync("const value = 1;", {
          ...options,
          plugins: [{
            name: "source-type",
            visitor: { Program(program) { actual = program.sourceType; } },
          }],
        });
        return actual;
      }

      assert.equal(sourceType({}), "module");
      assert.equal(sourceType({ filename: "input.cjs" }), "commonjs");
      assert.equal(sourceType({ filename: "input.cts" }), "commonjs");
      assert.equal(sourceType({ filename: "input.mjs" }), "module");
      assert.equal(sourceType({ filename: "input.mts" }), "module");
      assert.equal(sourceType({ filename: "input.ts" }), "script");
      assert.equal(sourceType({ parser: { sourceType: "module" } }), "module");
      assert.equal(sourceType({ parser: { sourceType: "script" } }), "script");
      assert.equal(sourceType({ parser: { sourceType: "commonjs" } }), "commonjs");
      assert.equal(sourceType({ filename: "input.cjs", parser: { sourceType: "module" } }), "module");
      assert.equal(sourceType({ filename: "input.mjs", parser: { sourceType: "script" } }), "script");

      assertSameTransform("const value = () => 1;", { parser: {}, generator: {} });
    });

    test("transformSync forwards parser and transform options", () => {
      const seen = [];
      const result = transformSync("const value: number = (1);", {
        filename: "input.ts",
        transformer: { typescript: false },
        parser: { preserveParens: false },
        plugins: [{
          name: "inspect",
          stage: "beforeCodegen",
          visitor: {
            VariableDeclarator(node) { seen.push(node.init.type); },
            TSNumberKeyword() { seen.push("type"); },
          },
        }],
      });

      assert.deepEqual(seen, ["Literal", "type"]);
      assert.equal(result.code, "const value: number = 1;\n");

      function staticBlocks(options) {
        let remainingBlocks = 0;
        transformSync("class C { static { run(); } }", {
          transformer: { features: options },
          plugins: [{
            name: "inspect-static-blocks",
            stage: "beforeCodegen",
            visitor: { StaticBlock() { remainingBlocks++; } },
          }],
        });
        return remainingBlocks;
      }

      assert.equal(staticBlocks({}), 1);
      assert.equal(staticBlocks({ "class-static-block": true }), 0);
      assert.equal(staticBlocks({ "class-static-block": false }), 1);
    });

    test("transformSync forwards parentheses configuration to afterParse plugins", () => {
      const source = "const value = (1);";
      assert.equal(parseSync(source, { preserveParens: true }).program.body[0].declarations[0].init.type, "ParenthesizedExpression");
      for (const preserveParens of [false, true]) {
        let actual;
        const result = transformSync(source, {
          parser: { preserveParens },
          plugins: [{
            name: "inspect-parentheses",
            visitor: { VariableDeclarator(node) { actual = node.init.type; } },
          }],
        });
        assert.equal(actual, preserveParens ? "ParenthesizedExpression" : "Literal");
        assert.equal(result.code, "const value = 1;\n");
      }
    });

    test("transformSync honors a plugin's edited Program source type", () => {
      const result = transformSync("run();", {
        parser: { sourceType: "script", semanticErrors: true },
        plugins: [{
          name: "convert-to-module",
          visitor: {
            Program(program) {
              program.sourceType = "module";
              program.body.unshift({
                type: "ImportDeclaration",
                specifiers: [],
                source: { type: "Literal", value: "pkg", raw: '"pkg"' },
                attributes: [],
                importKind: "value",
              });
            },
          },
        }],
      });
      assert.equal(result.code, 'import "pkg";\nrun();\n');
      assert.deepEqual(result.diagnostics, []);
    });

    test("transformSync forwards module modes across plugin stage configurations", () => {
      assertSameTransform("with (scope) {}", { parser: { sourceType: "script" } });

      assert.ok(assertSameTransformError("with (scope) {}", { parser: { sourceType: "module", semanticErrors: true } }) instanceof SyntaxError);

      assertSameTransform("return new.target;", { parser: { sourceType: "commonjs" } });

      assert.ok(assertSameTransformError("return new.target;", { parser: { sourceType: "script" } }) instanceof SyntaxError);

      assert.ok(assertSameTransformError("return new.target;", { parser: { sourceType: "module" } }) instanceof SyntaxError);
    });

    test("transformSync maps output options across plugin stage configurations", () => {
      const options = {
        generator: { asciiOnly: true, minify: true, inlineScript: true },
      };
      const source = 'const value = "😀</script>";';
      const expected = 'const value="\\u{1F600}<\\/script>";';

      assert.equal(assertSameTransform(source, options).code, expected);

      assert.equal(assertSameTransform('const value = "😀";', {
        generator: { asciiOnly: false, minify: false },
      }).code, 'const value = "😀";\n');
    });

    test("transformSync forwards TypeScript emission flags across plugin stages", () => {
      const source = "const enum E { X = 1 } function read(E: { X: number }) { return E.X; } export const value = E.X;";
      const lang = "ts";
      const defaults = assertSameTransform(source, { parser: { lang } });
      assert.deepEqual(assertSameTransform(source, {
        parser: { lang, semanticErrors: false },
      }), defaults);
      assert.deepEqual(assertSameTransform(source, {
        parser: { lang, semanticErrors: true },
      }), defaults);
      assert.ok(!defaults.code.includes("var E"));
      assert.ok(defaults.code.includes("export const value = 1"));
      assert.ok(defaults.code.includes("return E.X"));

      const preserved = assertSameTransform(source, {
        parser: { lang },
        transformer: { typescript: { preserveConstEnums: true } },
      });
      assert.ok(preserved.code.includes("var E"));
      assert.ok(preserved.code.includes("export const value = 1"));

      const verbatim = assertSameTransform(source, {
        parser: { lang },
        transformer: { typescript: { verbatimModuleSyntax: true } },
      });
      assert.ok(verbatim.code.includes("var E"));
      assert.ok(verbatim.code.includes("export const value = E.X"));

      const imports = assertSameTransform('import { value } from "dependency";', {
        parser: { lang },
        transformer: { typescript: { verbatimModuleSyntax: true } },
      });
      assert.ok(imports.code.includes('import { value } from "dependency"'));
    });

    test("transformSync uses Rust TypeScript defaults and supports disabling the pass", () => {
      const source = "const value: number = 1;";
      const options = { filename: "input.ts" };
      const inferred = assertSameTransform(source, options);
      const enabled = assertSameTransform(source, { ...options, transformer: { typescript: {} } });
      assert.deepEqual(enabled, inferred);
      assert.deepEqual(assertSameTransform(source, { ...options, transformer: { typescript: true } }), inferred);
      assert.equal(enabled.code, "const value = 1;\n");

      const disabled = assertSameTransform(source, { ...options, transformer: { typescript: false } });
      assert.equal(disabled.code, "const value: number = 1;\n");

      const module = assertSameTransform(source, { filename: "input.mts" });
      assert.equal(module.code, "const value = 1;\nexport {};\n");
    });

    test("transformSync forwards JSX runtime and import options across plugin stages", () => {
      const source = 'import { h, Fragment } from "ui"; export const view: unknown = <><section /></>;';
      const options = { filename: "input.tsx", comments: "none" };
      const classic = assertSameTransform(source, {
        ...options,
        transformer: { jsx: { runtime: "classic", pragma: "h", pragmaFrag: "Fragment", pure: false } },
      });
      const { program } = parseSync(classic.code);
      assert.deepEqual(program.body[0].specifiers.map(specifier => specifier.local.name), ["h", "Fragment"]);
      assert.ok(classic.code.includes("h(Fragment"));
      assert.ok(!classic.code.includes("<section"));

      const automatic = assertSameTransform("export const view = <section />;", {
        filename: "input.jsx",
        comments: "none",
        transformer: { jsx: { importSource: "preact", pure: false } },
      });
      assert.ok(automatic.code.includes('"preact/jsx-runtime"'));
    });

    test("transformSync keeps JSX pragmas out of unsupported AST transfer paths", () => {
      const source = "/** @jsxRuntime classic */ export const view = <section />;";
      const options = { filename: "input.jsx", comments: "none", transformer: { jsx: { pure: false } } };
      const expected = transformSync(source, options);
      assert.ok(expected.code.includes("React.createElement"));
      assert.ok(!expected.code.includes("react/jsx-runtime"));
      assert.deepEqual(transformSync(source, {
        ...options,
        plugins: [afterPlugin],
      }), expected);

      let visited = false;
      assert.throws(() => transformSync(source, {
        ...options,
        plugins: [{ name: "must-not-run", visitor: { Program() { visited = true; } } }],
      }), error => {
        assert.ok(error instanceof TypeError);
        assert.match(error.message, /pragma/);
        return true;
      });
      assert.equal(visited, false);

      assertSameTransform("/* ordinary comment */ export const view = <section />;", options);

      assertSameTransform("const x = 1;\n// Example: `@jsx h`\nconst view = <div />;", options);

      assertSameTransform("/** Example: `@jsx h` */ export const view = <section />;", options);

      assertSameTransform("/** @jsxFactory h */ export const view = <section />;", options);
    });

    test("transformSync forwards JSX development filename and guards source-dependent plugins", () => {
      const source = "export const view = <section />;";
      const options = {
        filename: "src/view.jsx",
        comments: "none",
        transformer: { jsx: { development: true, sourceSelf: true, pure: false } },
      };
      const expected = transformSync(source, options);
      assert.ok(expected.code.includes("src/view.jsx"));
      assert.ok(expected.code.includes("jsxDEV"));
      assert.deepEqual(transformSync(source, {
        ...options,
        plugins: [afterPlugin],
      }), expected);

      function unsupported(jsx) {
        let visited = false;
        assert.throws(() => transformSync(source, {
          filename: "input.jsx",
          transformer: { jsx },
          plugins: [{ name: "must-not-run", visitor: { Program() { visited = true; } } }],
        }), TypeError);
        assert.equal(visited, false);
      }
      unsupported({ development: true, sourceSelf: true });
      unsupported({ refresh: true });
      unsupported({ refresh: {} });
    });

    test("transformSync normalizes React Refresh toggles before the Rust boundary", () => {
      const source = "export function App() { return <section />; }";
      const options = { filename: "input.jsx", comments: "none" };
      const jsx = { development: true, pure: false };
      const enabled = transformSync(source, {
        ...options,
        transformer: { jsx: { ...jsx, refresh: true } },
      });
      assert.deepEqual(transformSync(source, {
        ...options,
        transformer: { jsx: { ...jsx, refresh: true } }, plugins: [afterPlugin],
      }), enabled);

      const disabled = assertSameTransform(source, {
        ...options,
        transformer: { jsx: { ...jsx, refresh: false } },
      });
      assert.ok(!disabled.code.includes("$RefreshReg$"));
    });

    test("transformSync preserves JSX transform diagnostics across plugin stages", () => {
      const source = "export const view = <svg:path />;";
      const error = assertSameTransformError(source, {
        filename: "input.jsx", comments: "none", parser: { semanticErrors: false },
      });
      assert.ok(error instanceof SyntaxError);
      assert.ok(error.diagnostics.some(diagnostic => diagnostic.severity === "error"));

      const result = assertSameTransform(source, {
        filename: "input.jsx",
        comments: "none",
        transformer: { jsx: { throwIfNamespace: false, pure: false } },
      });
      assert.ok(result.code.includes('"svg:path"'));
    });

    test("transformSync forwards per-feature lowering overrides", () => {
      const source = "try { run(); } catch { recover(); }";
      const defaults = assertSameTransform(source);
      assert.ok(defaults.code.includes("catch {"));

      const lowered = assertSameTransform(source, {
        transformer: { features: { "optional-catch-binding": true } },
      });
      assert.match(lowered.code, /catch \(\w+\)/);

      assert.deepEqual(assertSameTransform(source, {
        transformer: { features: { "optional-catch-binding": false } },
      }), defaults);
    });

    test("transformSync forwards canonical Rust configuration across plugin stages", () => {
      const source = "/* discard */ const value = 1;";
      const options = {
        comments: "none",
        parser: {},
        transformer: { typescript: null, assumptions: {}, proposal: {} },
        generator: {},
      };
      assert.equal(assertSameTransform(source, options).code, "const value = 1;\n");
      assert.equal(assertSameTransform("const value = 1;", {
        generator: { minify: true },
      }).code, "const value=1;");
    });

    test("transformSync rejects minify before visitors and preserves the uncompressed AST", () => {
      const source = "export const value = 1 + 2;";
      assertSameTransformError(source, { minify: true });
      assertSameTransformError(source, { minify: false });
      assertSameTransformError(source, { minify: {} });
      let initializer;
      transformSync(source, {
        generator: { minify: true },
        plugins: [{
          name: "inspect-transformed",
          stage: "beforeCodegen",
          visitor: {
            VariableDeclarator(node) { initializer = node.init; },
          },
        }],
      });
      assert.equal(initializer.type, "BinaryExpression");
      assert.equal(initializer.operator, "+");
    });

    test("transformSync supports source maps when the plugin list is empty", () => {
      const source = "export const value: number = 1;";
      const options = { filename: "input.ts", sourceMap: true };
      const result = transformSync(source, options);
      assert.deepEqual(transformSync(source, { ...options, plugins: [] }), result);
      assert.equal(JSON.parse(result.map).version, 3);
    });

    test("transformSync preserves string values through both plugin boundaries", () => {
      const original = "é😀\0\uD800";
      const before = `${original}前`;
      const after = `${before}後\uDC00`;
      const result = transformSync('const value = "é😀\\0\\uD800";', {
        plugins: [{
          name: "before-strings",
          visitor: {
            Literal(node) {
              assert.equal(node.value, original);
              node.value = before;
            },
          },
        }, {
          name: "after-strings",
          stage: "beforeCodegen",
          visitor: {
            Literal(node) {
              assert.equal(node.value, before);
              node.value = after;
            },
          },
        }],
      });

      assert.equal(parseSync(result.code).program.body[0].declarations[0].init.value, after);
    });

    test("transformSync reports UTF-16 diagnostic positions before invoking plugins", () => {
      const source = "/* 😀 */ const = 1;";
      for (const stage of ["afterParse", "beforeCodegen"]) {
        let visited = false;

        assert.throws(() => transformSync(source, {
          filename: "broken.js",
          comments: "none",
          plugins: [{
            name: "must-not-run",
            stage,
            visitor: { Program() { visited = true; } },
          }],
        }), (error) => {
          assert.ok(error instanceof SyntaxError);
          assert.match(error.message, /broken\.js/);
          assert.equal(error.diagnostics[0].severity, "error");
          assert.equal(error.diagnostics[0].start, source.indexOf("="));
          assert.equal(error.diagnostics[0].end, source.indexOf("=") + 1);
          return true;
        });
        assert.equal(visited, false);
      }
    });

    test("transformSync preserves original UTF-16 spans in after plugins", () => {
      const source = "/* 😀 */ const 𐐀 = 1;";
      const start = source.indexOf("𐐀");
      let visited = false;
      const result = transformSync(source, {
        comments: "none",
        plugins: [{
          name: "inspect-spans",
          stage: "beforeCodegen",
          visitor: {
            Identifier(node) {
              visited = true;
              assert.equal(node.name, "𐐀");
              assert.equal(node.start, start);
              assert.equal(node.end, start + "𐐀".length);
            },
          },
        }],
      });

      assert.equal(visited, true);
      assert.deepEqual(result.diagnostics, []);
    });

    test("transformSync controls semantic errors introduced by before plugins", () => {
      const source = "/* 😀 */ let 文;";
      const duplicate = parseSync(source).program.body[0];
      const duplicatePlugin = {
        name: "duplicate-declaration",
        visitor: {
          VariableDeclaration(_, ctx) { ctx.insertBefore(duplicate); },
        },
      };
      for (const includeAfter of [false, true]) {
        let visitedAfter = false;
        const plugin = [duplicatePlugin];
        if (includeAfter) plugin.push({
          name: "must-not-run-after",
          stage: "beforeCodegen",
          visitor: { Program() { visitedAfter = true; } },
        });

        const options = { filename: "duplicate.js", comments: "none", plugins: plugin };
        const disabled = transformSync(source, {
          ...options,
          parser: { semanticErrors: false },
        });
        assert.deepEqual(disabled, { code: "let 文;\nlet 文;\n", diagnostics: [] });
        assert.equal(visitedAfter, includeAfter);

        visitedAfter = false;
        assert.deepEqual(transformSync(source, options), disabled);
        assert.equal(visitedAfter, includeAfter);

        visitedAfter = false;
        assert.throws(() => transformSync(source, {
          ...options,
          parser: { semanticErrors: true },
        }), (error) => {
          assert.ok(error instanceof SyntaxError);
          assert.match(error.message, /duplicate\.js/);
          const diagnostic = error.diagnostics.find((diagnostic) => diagnostic.severity === "error");
          assert.equal(diagnostic.start, source.indexOf("文"));
          assert.equal(diagnostic.end, source.indexOf("文") + 1);
          assert.ok(diagnostic.labels.some((label) => label.start === source.indexOf("文")));
          return true;
        });
        assert.equal(visitedAfter, false);
      }
    });

    test("transformSync validates plugin configuration", () => {
      const source = "const value = 1;";

      assert.throws(() => transformSync(source, { plugins: {} }), TypeError);
      assert.throws(() => transformSync(source, { plugins: [null] }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "old-stage", phase: "before", visitor: {} }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "old-hooks", visitors: {} }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ visitor: {} }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "", visitor: {} }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "invalid-stage", stage: "pre", visitor: {} }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "invalid-visitor", visitor: null }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "invalid-hook", visitor: { UnknownNode() {} } }],
      }), TypeError);
    });

    test("transformSync rejects asynchronous and non-undefined hook results", () => {
      const source = "const value = 1;";

      assert.throws(() => transformSync(source, {
        plugins: [{ name: "async", visitor: { async Program() {} } }],
      }), TypeError);
      assert.throws(() => transformSync(source, {
        plugins: [{ name: "return-value", visitor: { Program() { return null; } } }],
      }), TypeError);
    });

    test("transformSync propagates plugin errors and keeps calls independent", () => {
      const failure = new Error("plugin failed");
      for (const stage of ["afterParse", "beforeCodegen"]) {
        assert.throws(() => transformSync("const value = 1;", {
          plugins: [{
            name: "throwing",
            stage,
            visitor: { Identifier() { throw failure; } },
          }],
        }), (error) => error === failure);
      }

      const plugin = {
        name: "rename",
        visitor: { Identifier(node) { node.name = "changed"; } },
      };
      const options = { plugins: [plugin] };
      assert.equal(transformSync("const first = 1;", options).code, "const changed = 1;\n");
      assert.equal(transformSync("const second = 2;", options).code, "const changed = 2;\n");
      assert.equal(transformSync("const original = 3;").code, "const original = 3;\n");
    });
  });
}
