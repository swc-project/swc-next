import assert from "node:assert/strict";
import { registerHooks } from "node:module";
import { randomUUID } from "node:crypto";
import test, { describe } from "node:test";
import { transformSync as core, parseSync } from "@swc-next/core";
import { transformSync as standalone } from "@swc-next/transformer";

for (const [name, transformSync] of [["core", core], ["transformer", standalone]]) {
  describe(`${name} transform plugins`, () => {
    const before = { name: "before", visitor: {} };
    const after = { name: "after", stage: "beforeCodegen", visitor: {} };
    const stages = [[before], [after], [before, after]];
    const options = { comments: "none", parser: { lang: "ts" } };

    test("empty plugins preserve the current source-only API", () => {
      for (const source of ["const x: number = 1;", "const = 1;"]) {
        assert.deepEqual(transformSync(source, { ...options, plugins: [] }), transformSync(source, options));
      }
      const source = "const x = 1;";
      assert.deepEqual(transformSync(source, { sourceMap: true, plugins: [] }), transformSync(source, { sourceMap: true }));
    });

    test("noop plugin stages retain current parser and generator options", () => {
      for (const plugins of stages) {
        for (const source of ["const x: number = 1;", "const x = 9007199254740993n;", 'const 文 = "é😀\\0\\uD800";']) {
          const config = { ...options, generator: { asciiOnly: true, minify: true } };
          assert.deepEqual(transformSync(source, { ...config, plugins }), transformSync(source, config));
        }
      }
    });

    test("visitors see the full Program and can remove or replace debugger", () => {
      for (const stage of ["afterParse", "beforeCodegen"]) {
        const imports = [];
        const output = transformSync('import value from "pkg"; debugger; if (value) debugger;', {
          comments: "none",
          plugins: [{ name: "remove-debugger", stage, visitor: {
            Program(program) { imports.push(...program.body.filter(node => node.type === "ImportDeclaration").map(node => node.source.value)); },
            DebuggerStatement(_node, ctx) { if (ctx.index !== null) ctx.remove(); else ctx.replace({ type: "EmptyStatement" }); },
          } }],
        });
        assert.deepEqual(imports, ["pkg"]);
        assert.ok(!output.code.includes("debugger"));
        assert.equal(parseSync(output.code).diagnostics.length, 0);
      }
    });

    test("source-backed literals and TypeScript namespaces survive noop transfers", () => {
      for (const plugins of stages) {
        for (const source of [
          'const value = /ab+/gi;',
          'const x = `a\r\nb${1}`;',
          'namespace N.M { export const x = 1; }',
          'namespace N { export let x = 0; ({ x } = { x: 1 }); }',
          'import foo = require("pkg"); console.log(foo);',
        ]) {
          assert.deepEqual(transformSync(source, { ...options, plugins }), transformSync(source, options));
        }
      }
    });

    test("each plugin completes a walk in stage array order", () => {
      const events = [];
      const plugin = (name, stage) => ({ name, stage, visitor: {
        Program() { events.push(`${name}:Program`); },
        Identifier() { events.push(`${name}:Identifier`); },
      } });
      transformSync("let value;", { plugins: [plugin("after", "beforeCodegen"), plugin("one", "afterParse"), plugin("two", "afterParse")] });
      assert.deepEqual(events, ["one:Program", "one:Identifier", "two:Program", "two:Identifier", "after:Program", "after:Identifier"]);
    });

    test("parser comment collection cannot override output comments", () => {
      const source = "/* comment */ let value;";
      for (const plugins of stages) {
        assert.ok(!transformSync(source, { comments: "none", parser: { comments: "all" }, plugins }).code.includes("comment"));
        assert.ok(!transformSync(source, { comments: "all", parser: { comments: "none" }, plugins }).code.includes("comment"));
      }
    });

    test("unsupported metadata and invalid options fail before callbacks", () => {
      let calls = 0;
      const plugins = [{ name: "observe", visitor: { Program() { calls++; } } }];
      assert.throws(() => transformSync("let x;", { sourceMap: true, plugins }), TypeError);
      assert.throws(() => transformSync("/* keep */ let x;", { plugins }), TypeError);
      assert.throws(() => transformSync("let x;", { parser: { lang: "invalid" }, plugins }), TypeError);
      assert.throws(() => transformSync("let x;", { plugins: [{ name: "invalid", stage: "before", visitor: {} }] }), TypeError);
      assert.equal(calls, 0);
    });

    test("invalid source errors carry UTF-16 positions before visitors", () => {
      let calls = 0;
      const source = "/* 😀 */ const = 1;";
      assert.throws(() => transformSync(source, { comments: "none", plugins: [{ name: "observe", visitor: { Program() { calls++; } } }] }), error => {
        assert.ok(error instanceof SyntaxError);
        assert.equal(error.diagnostics[0].start, source.indexOf("="));
        return true;
      });
      assert.equal(calls, 0);
    });

    test("afterParse semantic checks honor the edited tree and UTF-16 labels", () => {
      const source = "/* 😀 */ let 文;";
      const plugins = [{ name: "duplicate", visitor: { Program(program) { program.body.push(structuredClone(program.body[0])); } } }];
      assert.throws(() => transformSync(source, { comments: "none", parser: { semanticErrors: true }, plugins }), error => {
        assert.ok(error instanceof SyntaxError);
        assert.equal(error.diagnostics[0].start, source.indexOf("文"));
        return true;
      });
      assert.equal(transformSync(source, { comments: "none", plugins }).diagnostics.length, 0);
    });

    test("edited declaration files retain their configured semantic context", () => {
      const assignment = parseSync("export = 1 + 2;", { lang: "ts" }).program.body[0];
      const plugins = [{ name: "assignment", visitor: { Program(program) { program.body.push(assignment); } } }];
      assert.throws(() => transformSync("declare const value: number;", {
        filename: "input.d.ts", parser: { semanticErrors: true }, plugins,
      }), SyntaxError);
      assert.equal(transformSync("const value: number = 0;", {
        filename: "input.ts", parser: { semanticErrors: true }, plugins,
      }).diagnostics.length, 0);
    });

    test("beforeCodegen edits print without another semantic check", () => {
      const plugins = [{ name: "duplicate", stage: "beforeCodegen", visitor: { Program(program) { program.body.push(structuredClone(program.body[0])); } } }];
      const output = transformSync("let x;", { parser: { semanticErrors: true }, plugins });
      assert.equal(output.code, "let x;\nlet x;\n");
    });

    test("Program.sourceType edits reach semantic checks", () => {
      const imported = parseSync('import "pkg";').program.body[0];
      const output = transformSync("run();", { parser: { sourceType: "script", semanticErrors: true }, plugins: [{ name: "module", visitor: { Program(program) { program.sourceType = "module"; program.body.unshift(imported); } } }] });
      assert.ok(output.code.includes('import "pkg"'));
    });

    test("plugin errors propagate and hooks must synchronously return undefined", () => {
      const failure = new Error("plugin failed");
      assert.throws(() => transformSync("", { plugins: [{ name: "failure", visitor: { Program() { throw failure; } } }] }), error => error === failure);
      assert.throws(() => transformSync("", { plugins: [{ name: "async", visitor: { async Program() {} } }] }), TypeError);
      assert.throws(() => transformSync("", { plugins: [{ name: "result", visitor: { Program() { return 1; } } }] }), TypeError);
    });

    test("nested transforms cannot overwrite the outer option buffer", () => {
      const output = transformSync("let x;", { generator: { minify: true }, plugins: [{ name: "nested", visitor: { Program() { transformSync("let y;", { filename: "nested.js", generator: { minify: false } }); } } }] });
      assert.equal(output.code, "let x;");
    });
  });
}

test("empty plugin lists never encode or decode ASTs", async () => {
  const id = `swc-plugin-test:${randomUUID()}`;
  const subject = new URL(`../../../packages/protocol/plugins.js?${randomUUID()}`, import.meta.url).href;
  const hooks = registerHooks({
    resolve(specifier, context, next) {
      if (context.parentURL === subject && (specifier === "./ast/encode.js" || specifier === "./ast/decode.js")) return { url: `${id}/${specifier}`, shortCircuit: true };
      return next(specifier, context);
    },
    load(url, context, next) {
      if (url.startsWith(id)) return { format: "module", source: 'export function encode() { throw new Error("AST transferred"); } export function decode() { throw new Error("AST transferred"); }', shortCircuit: true };
      return next(url, context);
    },
  });
  try {
    const { transformWithPlugins } = await import(subject);
    const result = transformWithPlugins("let x;", { plugins: [] }, { transformSync() { return "let x;\n"; } });
    assert.deepEqual(result, { code: "let x;\n", diagnostics: [] });
  } finally { hooks.deregister(); }
});
