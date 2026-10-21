import assert from "node:assert/strict";
import test from "node:test";

import { parseSync, transformSync } from "../src-js/index.js";

test("transformSync exposes the complete program and supports statement edits", () => {
  const imports = [];
  const source = 'import value from "dependency"; debugger; if (value) debugger;';
  const result = transformSync(source, {
    filename: "input.js",
    jsPlugins: [{
      name: "remove-debugger",
      visitors: {
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
    jsPlugins: [{
      name: "before",
      visitors: {
        Program(program) { beforeProgram = program; },
        Identifier(node) {
          assert.equal(node.typeAnnotation.type, "TSTypeAnnotation");
          node.name = "beforeValue";
        },
        TSNumberKeyword() { seen.push("type"); },
      },
    }, {
      name: "after",
      phase: "after",
      visitors: {
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

test("transformSync runs a complete walk per plugin in phase order", () => {
  const seen = [];
  function rename(name, phase, from, to) {
    return {
      name,
      phase,
      visitors: {
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
    jsPlugins: [
      rename("after-first", "after", "second", "third"),
      rename("before-first", "before", "original", "first"),
      rename("after-second", "after", "third", "finalValue"),
      rename("before-second", "before", "first", "second"),
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

test("transformSync respects statement order changed by a before plugin", () => {
  const options = { filename: "input.ts" };
  const expected = transformSync("enum E { Member = value } const value = 1;", options);
  const result = transformSync("const value = 1; enum E { Member = value }", {
    ...options,
    jsPlugins: [{
      name: "reorder-statements",
      visitors: { Program(program) { program.body.reverse(); } },
    }],
  });

  assert.equal(result.code, expected.code);
});

test("transformSync infers parser options from filename and honors overrides", () => {
  assert.equal(
    transformSync("const value: number = 1;", { filename: "input.ts" }).code,
    "const value = 1;\n",
  );

  assert.equal(
    transformSync("const value: number = 1;", {
      filename: "input.js",
      parser: { lang: "ts" },
    }).code,
    "const value = 1;\n",
  );

  function sourceType(options) {
    let actual;
    transformSync("const value = 1;", {
      ...options,
      jsPlugins: [{
        name: "source-type",
        visitors: { Program(program) { actual = program.sourceType; } },
      }],
    });
    return actual;
  }

  assert.equal(sourceType({}), "module");
  assert.equal(sourceType({ filename: "input.cjs" }), "commonjs");
  assert.equal(sourceType({
    filename: "input.cjs",
    parser: { sourceType: "module" },
  }), "module");
});

test("transformSync forwards parser and transform options", () => {
  const seen = [];
  const result = transformSync("const value: number = (1);", {
    filename: "input.ts",
    parser: { preserveParens: false },
    transform: { typescript: false },
    jsPlugins: [{
      name: "inspect",
      phase: "after",
      visitors: {
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
      transform: options,
      jsPlugins: [{
        name: "inspect-static-blocks",
        phase: "after",
        visitors: { StaticBlock() { remainingBlocks++; } },
      }],
    });
    return remainingBlocks;
  }

  assert.equal(staticBlocks({}), 1);
  assert.equal(staticBlocks({ staticBlocks: true }), 0);
});

test("transformSync forwards codegen options with and without plugins", () => {
  const codegen = { asciiOnly: true, inlineScript: true, minify: true };
  const source = 'const value = "😀</script>";';
  const expected = 'const value="\\u{1F600}<\\/script>";';

  assert.equal(transformSync(source, { codegen }).code, expected);
  assert.equal(transformSync(source, {
    codegen,
    jsPlugins: [{ name: "empty", phase: "after", visitors: {} }],
  }).code, expected);
});

test("transformSync preserves string values through both plugin boundaries", () => {
  const original = "é😀\0\uD800";
  const before = `${original}前`;
  const after = `${before}後\uDC00`;
  const result = transformSync('const value = "é😀\\0\\uD800";', {
    jsPlugins: [{
      name: "before-strings",
      visitors: {
        Literal(node) {
          assert.equal(node.value, original);
          node.value = before;
        },
      },
    }, {
      name: "after-strings",
      phase: "after",
      visitors: {
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
  let visited = false;

  assert.throws(() => transformSync(source, {
    filename: "broken.js",
    jsPlugins: [{
      name: "must-not-run",
      visitors: { Program() { visited = true; } },
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
});

test("transformSync preserves original UTF-16 spans in after plugins", () => {
  const source = "/* 😀 */ const 𐐀 = 1;";
  const start = source.indexOf("𐐀");
  let visited = false;
  const result = transformSync(source, {
    jsPlugins: [{
      name: "inspect-spans",
      phase: "after",
      visitors: {
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

test("transformSync reports semantic errors introduced by before plugins", () => {
  const source = "let value;";
  const duplicate = parseSync(source).program.body[0];
  let visitedAfter = false;

  assert.throws(() => transformSync(source, {
    filename: "duplicate.js",
    jsPlugins: [{
      name: "duplicate-declaration",
      visitors: {
        VariableDeclaration(_, ctx) { ctx.insertBefore(duplicate); },
      },
    }, {
      name: "must-not-run-after",
      phase: "after",
      visitors: { Program() { visitedAfter = true; } },
    }],
  }), (error) => {
    assert.ok(error instanceof SyntaxError);
    assert.match(error.message, /duplicate\.js/);
    assert.ok(error.diagnostics.some((diagnostic) => diagnostic.severity === "error"));
    return true;
  });
  assert.equal(visitedAfter, false);
});

test("transformSync validates plugin configuration", () => {
  const source = "const value = 1;";

  assert.throws(() => transformSync(source, { jsPlugins: {} }), TypeError);
  assert.throws(() => transformSync(source, { jsPlugins: [null] }), TypeError);
  assert.throws(() => transformSync(source, {
    jsPlugins: [{ visitors: {} }],
  }), TypeError);
  assert.throws(() => transformSync(source, {
    jsPlugins: [{ name: "", visitors: {} }],
  }), TypeError);
  assert.throws(() => transformSync(source, {
    jsPlugins: [{ name: "invalid-phase", phase: "pre", visitors: {} }],
  }), TypeError);
  assert.throws(() => transformSync(source, {
    jsPlugins: [{ name: "invalid-visitors", visitors: null }],
  }), TypeError);
  assert.throws(() => transformSync(source, {
    jsPlugins: [{ name: "invalid-hook", visitors: { UnknownNode() {} } }],
  }), TypeError);
});

test("transformSync rejects asynchronous and non-undefined hook results", () => {
  const source = "const value = 1;";

  assert.throws(() => transformSync(source, {
    jsPlugins: [{ name: "async", visitors: { async Program() {} } }],
  }), TypeError);
  assert.throws(() => transformSync(source, {
    jsPlugins: [{ name: "return-value", visitors: { Program() { return null; } } }],
  }), TypeError);
});

test("transformSync propagates plugin errors and keeps calls independent", () => {
  const failure = new Error("plugin failed");
  for (const phase of ["before", "after"]) {
    assert.throws(() => transformSync("const value = 1;", {
      jsPlugins: [{
        name: "throwing",
        phase,
        visitors: { Identifier() { throw failure; } },
      }],
    }), (error) => error === failure);
  }

  const plugin = {
    name: "rename",
    visitors: { Identifier(node) { node.name = "changed"; } },
  };
  const options = { jsPlugins: [plugin] };
  assert.equal(transformSync("const first = 1;", options).code, "const changed = 1;\n");
  assert.equal(transformSync("const second = 2;", options).code, "const changed = 2;\n");
  assert.equal(transformSync("const original = 3;").code, "const original = 3;\n");
});
