import assert from "node:assert/strict";
import { registerHooks } from "node:module";
import { randomUUID } from "node:crypto";
import { runInNewContext } from "node:vm";
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
      assert.deepEqual(transformSync("value?.prop;", { transformer: { target: "es2019" }, plugins: [] }),
        transformSync("value?.prop;", { transformer: { target: "es2019" } }));
      const inherited = Object.create({ filename: "input.ts" });
      const expected = transformSync("let x: number;", inherited);
      assert.equal(expected.code, "let x;\n");
      assert.deepEqual(transformSync("let x: number;", Object.assign(inherited, { plugins: [] })), expected);
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
        ]) {
          assert.deepEqual(transformSync(source, { ...options, plugins }), transformSync(source, options));
        }
        const source = 'import foo = require("pkg"); console.log(foo);';
        const config = { ...options, transformer: { module: { type: "commonjs" } } };
        assert.deepEqual(transformSync(source, { ...config, plugins }), transformSync(source, config));
      }
    });

    test("native transforms run between plugin stages and honor transformer options", () => {
      const seen = [];
      const output = transformSync("export const x: number = 2 ** 3;", {
        filename: "input.ts",
        transformer: { target: "es2015", module: { type: "commonjs" } },
        plugins: [
          { name: "before", visitor: {
            TSTypeAnnotation() { seen.push("type"); },
            Literal(node) { if (node.value === 2) node.value = 4; },
          } },
          { name: "after", stage: "beforeCodegen", visitor: {
            TSTypeAnnotation() { assert.fail("types must already be erased"); },
            Identifier(node) { if (node.name === "pow") seen.push("lowered"); },
          } },
        ],
      });
      assert.deepEqual(seen, ["type", "lowered"]);
      assert.match(output.code, /Math\.pow\(4, 3\)/);
      assert.match(output.code, /Object\.defineProperty\(exports, "x"/);
      assert.deepEqual(output.diagnostics, []);

      const disabled = { filename: "input.ts", transformer: { typescript: false } };
      assert.deepEqual(transformSync("const x: number = 1;", { ...disabled, plugins: [before, after] }),
        transformSync("const x: number = 1;", disabled));
    });

    test("transform diagnostics stop the beforeCodegen stage", () => {
      const source = 'import foo = require("pkg"); console.log(foo);';
      const config = { filename: "input.ts" };
      const expected = transformSync(source, config).diagnostics;
      assert.equal(expected[0].severity, "error");
      const observer = { ...after, visitor: { Program() { assert.fail("beforeCodegen must not run"); } } };
      for (const plugins of [[before], [observer], [before, observer]]) {
        assert.throws(() => transformSync(source, { ...config, plugins }), error => {
          assert.ok(error instanceof SyntaxError);
          assert.deepEqual(error.diagnostics, expected);
          return true;
        });
      }
    });

    test("JSX lowering preserves source locations, pragmas, and pure annotations", () => {
      const source = '/* @jsxRuntime classic @jsx h */\nconst label = "😀"; const view = <div />;';
      const config = {
        filename: "input.jsx",
        comments: "none",
        transformer: { jsx: { development: true, sourceSelf: true } },
      };
      const expected = transformSync(source, config);
      assert.match(expected.code, /h\("div"/);
      assert.match(expected.code, /lineNumber: 2/);
      assert.match(expected.code, /columnNumber: 34/);
      for (const plugins of stages) {
        assert.deepEqual(transformSync(source, { ...config, plugins }), expected);
      }
      const pure = { filename: "input.jsx" };
      const jsx = "const view = <div />;";
      const expectedPure = transformSync(jsx, pure);
      assert.match(expectedPure.code, /#__PURE__/);
      assert.deepEqual(transformSync(jsx, { ...pure, plugins: [before, after] }), expectedPure);
    });

    test("plugins share one walk per stage with callbacks in array order", () => {
      const events = [];
      const plugin = (name, stage) => ({ name, stage, visitor: {
        Program() { events.push(`${name}:Program`); },
        Identifier() { events.push(`${name}:Identifier`); },
      } });
      transformSync("let value;", { plugins: [plugin("after", "beforeCodegen"), plugin("one", "afterParse"), plugin("two", "afterParse")] });
      assert.deepEqual(events, ["one:Program", "two:Program", "one:Identifier", "two:Identifier", "after:Program", "after:Identifier"]);
    });

    test("single visitors preserve alias ordering and snapshot mutable callback arrays", () => {
      for (const stage of ["afterParse", "beforeCodegen"]) {
        const run = (isMerged) => {
          const events = [];
          const record = (label) => (node) => { events.push(`${label}:${node.type}`); };
          const globalEnter = [record("global-enter")];
          const concrete = { enter: [record("function-enter")], leave: [record("function-leave")] };
          const visitor = {
            enter: globalEnter,
            leave: record("global-leave"),
            FunctionDeclaration: concrete,
            Function: { enter: record("alias-enter"), leave: record("alias-leave") },
            Program() {
              globalEnter.push(record("late-global"));
              concrete.enter.push(record("late-function"));
              visitor.Identifier = record("late-identifier");
            },
          };
          const plugins = [{ name: "observe", stage, visitor }];
          if (isMerged) plugins.push({ name: "noop", stage, visitor: {} });
          const output = transformSync("function f() {}", { plugins });
          return { events, output };
        };
        const single = run(false);
        assert.deepEqual(single, run(true));
        assert.ok(!single.events.some((event) => event.startsWith("late-")));
        assert.deepEqual(single.events.filter((event) => event.endsWith(":FunctionDeclaration")), [
          "global-enter:FunctionDeclaration", "alias-enter:FunctionDeclaration", "function-enter:FunctionDeclaration",
          "function-leave:FunctionDeclaration", "alias-leave:FunctionDeclaration", "global-leave:FunctionDeclaration",
        ]);
      }
    });

    test("later plugins see edits to the current node before its children", () => {
      const names = [];
      const output = transformSync("let value;", { plugins: [
        { name: "rename", visitor: { Identifier(node) { node.name = "edited"; } } },
        { name: "observe", visitor: { Identifier(node) { names.push(node.name); } } },
      ] });
      assert.deepEqual(names, ["edited"]);
      assert.equal(output.code, "let edited;\n");
    });

    test("stop ends the merged stage but the next stage still runs", () => {
      const events = [];
      transformSync("let value;", { plugins: [
        { name: "stop", visitor: { Program(_node, ctx) { events.push("stop"); ctx.stop(); } } },
        { name: "skipped", visitor: { Program() { events.push("skipped"); } } },
        { name: "after", stage: "beforeCodegen", visitor: { Program() { events.push("after"); } } },
      ] });
      assert.deepEqual(events, ["stop", "after"]);
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
      const source = "// c\r\n/* 😀 */ const = 1;";
      assert.throws(() => transformSync(source, { plugins: [{ name: "observe", visitor: { Program() { calls++; } } }] }), error => {
        assert.ok(error instanceof SyntaxError);
        assert.match(error.message, /^input\.js:2:16: /);
        assert.equal(error.diagnostics[0].start, source.indexOf("="));
        return true;
      });
      assert.equal(calls, 0);
    });

    test("afterParse semantic recheck treats destructuring targets as references", () => {
      const config = { parser: { semanticErrors: true } };
      const pluginConfig = { ...config, plugins: [before, after] };
      for (const source of [
        "let a; function f() { [a] = []; }",
        "let a; function f() { ({ key: [a = 1] } = {}); }",
        "let a; function f() { for ([...a] of []); }",
        "let a; const f = () => { ({ ...a } = {}); };",
      ]) {
        assert.deepEqual(transformSync(source, pluginConfig), transformSync(source, config));
      }
    });

    test("afterParse semantic recheck catches legacy literals in newly strict code", () => {
      const directive = parseSync('"use strict";').program.body[0];
      const plugins = [{ name: "strict", visitor: { Program(program) { program.body.unshift(directive); } } }];
      const config = { parser: { sourceType: "script", semanticErrors: true }, plugins };
      for (const literal of ["010", "09", '"\\1"']) {
        const source = `var n = ${literal};`;
        assert.throws(() => transformSync(source, config), error => {
          assert.ok(error instanceof SyntaxError);
          assert.equal(error.diagnostics[0].start, source.indexOf(literal));
          return true;
        });
      }
    });

    test("afterParse edits cannot change the semantic check options for the current call", () => {
      const parser = { semanticErrors: true };
      const plugins = [{ name: "mutate-options", visitor: {
        Program(program) {
          program.body.push(structuredClone(program.body[0]));
          parser.semanticErrors = false;
        },
      } }];
      assert.throws(() => transformSync("let x;", { parser, plugins }), SyntaxError);
    });

    test("accepts plain plugin descriptors and callbacks from a VM realm", () => {
      const plugin = runInNewContext('({ name: "rename", visitor: { Identifier(node) { node.name = "edited"; } } })');
      assert.equal(transformSync("let value;", { plugins: [plugin] }).code, "let edited;\n");
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

    test("nested plugin transforms restore the outer callback and source", () => {
      const source = 'const 文 = "😀";';
      const output = transformSync(source, { plugins: [
        { name: "nested", visitor: {
          Program() {
            const nested = transformSync("let inner;", { plugins: [{ name: "rename", visitor: {
              Identifier(node) { node.name = "renamed"; },
            } }] });
            assert.equal(nested.code, "let renamed;\n");
          },
        } },
        { name: "outer", stage: "beforeCodegen", visitor: {
          Identifier(node) {
            assert.equal(node.name, "文");
            node.name = "outer";
          },
        } },
      ] });
      assert.equal(output.code, 'const outer = "😀";\n');
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
    const { runTransformPipeline } = await import(subject);
    const result = runTransformPipeline("let x;", { plugins: [] }, { transformSync() { return "let x;\n"; } });
    assert.deepEqual(result, { code: "let x;\n", diagnostics: [] });
  } finally { hooks.deregister(); }
});
