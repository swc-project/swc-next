import assert from "node:assert/strict";
import test from "node:test";

import { AstView } from "@swc-next/ast2";
import * as core from "@swc-next/core/ffi2";
import { langFromPath, parseSync, sourceTypeFromPath, transformSync } from "@swc-next/core/ffi2";
import { parseSync as parseStandalone } from "@swc-next/parser/ffi2";

test("parser options use strings without enum value exports", () => {
  assert.equal("Lang" in core, false);
  assert.equal("SourceType" in core, false);
  assert.equal("CommentMode" in core, false);
  assert.equal(langFromPath("input.ts"), "ts");
  assert.equal(sourceTypeFromPath("input.cjs"), "commonjs");

  const result = parseSync("// comment\nconst value: number = 1;", {
    lang: "ts",
    sourceType: "script",
    comments: "none",
  });
  assert.ok(result instanceof AstView);
  assert.equal(result.meta.sourceType, "script");
  assert.deepEqual(result.comments, []);
  assert.equal(result.program.body.get(0).declarators.get(0).typeAnnotation.type, "TsTypeAnnotation");
});

test("decoder preserves nested types through the lazy program getter", () => {
  const depth = 600;
  const source = `type Value = string${"[]".repeat(depth)};`;
  const result = parseSync(source, { lang: "ts" });
  const program = result.program;
  let type = program.body.get(0).typeAnnotation;
  for (let index = 0; index < depth; index++) {
    assert.equal(type.type, "TsArrayType");
    type = type.elementType;
  }
  assert.equal(type.type, "TsStringKeyword");
  assert.ok(AstView.isSame(result.program, program));

  const tsx = parseSync(source, { lang: "tsx" });
  assert.deepEqual(tsx.materialize(), result.materialize());
});

test("parseSync exposes the program lazily", () => {
  const result = parseSync("const answer = 42;");
  const descriptor = Object.getOwnPropertyDescriptor(AstView.prototype, "program");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(descriptor.set, undefined);
  assert.equal(descriptor.enumerable, false);
  assert.equal(descriptor.configurable, true);

  const program = result.program;
  assert.equal(program.type, "Program");
  assert.equal(program.body.get(0).type, "VariableDeclaration");
  assert.ok(result.program.isSame(program));
});

test("parseSync normalizes source-backed lone surrogates", () => {
  for (const lang of ["js", "ts"]) {
    const comments = parseSync("/* 😀 */\n// \ud800", { lang }).comments;
    assert.equal(comments[0].value, " 😀 ");
    assert.equal(comments[1].value, " \ufffd");

    const lone = parseSync('"\ud800";', { lang }).program.directives.get(0);
    assert.equal(lone.expression.value, "\ufffd");
    assert.equal(lone.value, "\ufffd");

    const loneTemplate = parseSync("tag`\ud800`;", { lang }).program.body.get(0);
    assert.equal(loneTemplate.expression.quasi.quasis.get(0).raw, "\ufffd");
    assert.equal(loneTemplate.expression.quasi.quasis.get(0).cooked, "\ufffd");
  }
});

test("parseSync preserves lazy source data across buffer reuse", () => {
  const source = '/* 雪😀 */ const 名字 = "🌍";';
  for (const parse of [parseSync, parseStandalone]) {
    for (const lang of ["js", "ts"]) {
      const result = parse(source, { lang, comments: "all" });

      parse(" ".repeat(256), { lang });
      transformSync(" ".repeat(512));

      const declaration = result.program.body.get(0).declarators.get(0);
      assert.equal(declaration.id.name, "名字");
      assert.equal(declaration.init.value, "🌍");
      assert.equal(declaration.init.raw, '"🌍"');
      assert.equal(result.comments[0].value, " 雪😀 ");
    }
  }
});

test("parseSync normalizes lone surrogates in RegExp source", () => {
  for (const lang of ["js", "ts"]) {
    const regexp = parseSync("/\ud800/gi;", { lang }).program.body.get(0).expression;
    assert.equal(regexp.pattern, "\ufffd");
    assert.equal(regexp.flags, "gi");
  }
});

test("parseSync exposes flat comments lazily", () => {
  const source =
    '#!/usr/bin/env node\nconst emoji = "😀";\n// one 😀\n/* two */';
  const result = parseSync(source, { comments: "all" });
  const descriptor = Object.getOwnPropertyDescriptor(AstView.prototype, "comments");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(descriptor.set, undefined);
  assert.equal(descriptor.enumerable, false);
  assert.equal(descriptor.configurable, true);

  const lineStart = source.indexOf("// one 😀");
  const blockStart = source.indexOf("/* two */");
  assert.deepEqual(result.comments, [
    {
      type: "Line",
      value: " one 😀",
      start: lineStart,
      end: lineStart + "// one 😀".length,
    },
    {
      type: "Block",
      value: " two ",
      start: blockStart,
      end: blockStart + "/* two */".length,
    },
  ]);
  assert.deepEqual(result.comments, result.comments);
});

test("parseSync exposes HTML comments without delimiters", () => {
  const source = "<!-- open\n--> close\n";
  const result = parseSync(source, {
    comments: "all",
    sourceType: "script",
  });

  assert.deepEqual(result.comments, [
    { type: "Line", value: " open", start: 0, end: 9 },
    { type: "Line", value: " close", start: 10, end: 19 },
  ]);
});

test("parseSync defaults to flat comments and returns an empty array", () => {
  const result = parseSync("const answer = 42;");

  assert.deepEqual(result.comments, []);
  assert.deepEqual(result.comments, result.comments);
});

test("parseSync forwards shared comment policies", () => {
  for (const parse of [parseSync, parseStandalone]) {
    const omitted = parse("// omitted", { comments: "none" });
    assert.deepEqual(omitted.comments, []);

    const collected = parse("// collected", { comments: "all" });
    assert.deepEqual(collected.comments.map(comment => comment.value), [" collected"]);
  }
});

test("errorRecovery controls whether the first diagnostic is thrown", () => {
  for (const parse of [parseSync, parseStandalone]) {
    const source = "const a; const b;";
    const recovered = parse(source, { errorRecovery: true });
    assert.equal(recovered.diagnostics.length, 2);
    assert.ok(recovered.program.body.length > 0);
    for (const options of [undefined, null, {}, { errorRecovery: false }]) {
      assert.throws(() => parse(source, options), (error) => {
        assert.deepEqual(error, recovered.diagnostics[0]);
        return true;
      });
    }
    const semantic = "let a; let a;";
    const options = { semanticErrors: true, errorRecovery: true };
    const first = parse(semantic, options).diagnostics[0];
    assert.throws(() => parse(semantic, { semanticErrors: true }), (error) => {
      assert.deepEqual(error, first);
      return true;
    });
    assert.deepEqual(parse("let a;", { errorRecovery: true }).diagnostics, []);
  }
});

test("parseSync exposes parser diagnostics lazily", () => {
  const result = parseSync("const = 1", { errorRecovery: true });
  const descriptor = Object.getOwnPropertyDescriptor(AstView.prototype, "diagnostics");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(descriptor.set, undefined);
  assert.equal(descriptor.enumerable, false);
  assert.equal(result.diagnostics.length, 1);
  const [error] = result.diagnostics;
  assert.equal(error.severity, "error");
  assert.ok(error.message.length > 0);
  assert.equal(error.start, 6);
  assert.equal(error.end, 7);
  assert.deepEqual(error.labels, []);
  assert.deepEqual(result.diagnostics, result.diagnostics);
  assert.equal(result.program.type, "Program");
});

test("parseSync returns an empty diagnostics array for valid input", () => {
  const result = parseSync("const answer = 42;");

  assert.deepEqual(result.diagnostics, []);
  assert.deepEqual(result.diagnostics, result.diagnostics);

  const empty = parseSync("");
  assert.deepEqual(empty.diagnostics, []);
});

test("parseSync makes semantic early-error checking optional", () => {
  const source = "let value; let value;";
  assert.deepEqual(parseSync(source).diagnostics, []);
  assert.deepEqual(parseSync(source, { semanticErrors: false }).diagnostics, []);

  const checked = parseSync(source, { semanticErrors: true, errorRecovery: true });
  assert.equal(checked.diagnostics.length, 1);
  assert.equal(checked.diagnostics[0].severity, "error");
  assert.equal(checked.program.body.length, 2);
  assert.deepEqual(checked.materialize(), parseSync(source).materialize());

  const valid = parseSync("let value; { let value; }", { semanticErrors: true, errorRecovery: true });
  assert.deepEqual(valid.diagnostics, []);
  const missingExport = "export { missing };";
  assert.deepEqual(parseSync(missingExport).diagnostics, []);
  assert.equal(parseSync(missingExport, { semanticErrors: true, errorRecovery: true }).diagnostics.length, 1);
  assert.throws(() => parseSync(source, { semanticErrors: 1 }), TypeError);
});

test("semantic checking supports TypeScript through both parser entry points", () => {
  const source = "type Value = string; type Value = number;";
  const options = { lang: "ts", semanticErrors: true, errorRecovery: true };
  const checked = parseSync(source, options);
  assert.deepEqual(parseSync(source, { lang: "ts" }).diagnostics, []);
  assert.deepEqual(parseSync(source, { ...options, semanticErrors: false }).diagnostics, []);
  assert.equal(checked.diagnostics.length, 1);
  assert.equal(checked.diagnostics[0].severity, "error");
  assert.deepEqual(checked.materialize(), parseSync(source, { lang: "ts" }).materialize());
  assert.deepEqual(parseStandalone(source, options).diagnostics, checked.diagnostics);
  assert.deepEqual(parseStandalone(source, options).materialize(), checked.materialize());
  assert.deepEqual(parseStandalone(source, { ...options, semanticErrors: false }).diagnostics, []);

  const valid = parseSync("interface Value {} interface Value {}", options);
  assert.deepEqual(valid.diagnostics, []);

  const invalid = "type Value = ;";
  assert.deepEqual(
    parseSync(invalid, options).diagnostics,
    parseSync(invalid, { lang: "ts", errorRecovery: true }).diagnostics,
  );
});

test("semantic diagnostics preserve Unicode offsets and related labels", () => {
  const source = "/* 😀 */ let 文; let 文;";
  const result = parseSync(source, { semanticErrors: true, comments: "none", errorRecovery: true });
  const [error] = result.diagnostics;
  assert.equal(error.start, source.lastIndexOf("文"));
  assert.equal(error.end, source.lastIndexOf("文") + 1);
  assert.ok(error.labels.some((label) => label.start === source.indexOf("文")));
  assert.ok(error.message.includes("文"));
  assert.deepEqual(result.comments, []);
});

test("semantic checking appends diagnostics after parser recovery", () => {
  function checkRecovery(source, lang) {
    const unchecked = parseSync(source, { lang, errorRecovery: true });
    const checked = parseSync(source, { lang, semanticErrors: true, errorRecovery: true });
    assert.equal(unchecked.diagnostics.length, 1);
    assert.equal(checked.diagnostics.length, 2);
    assert.deepEqual(checked.diagnostics.slice(0, 1), unchecked.diagnostics);
    assert.equal(checked.diagnostics[1].severity, "error");
    assert.deepEqual(checked.program, unchecked.program);
    assert.deepEqual(checked.comments, unchecked.comments);
    assert.deepEqual(
      parseStandalone(source, { lang, semanticErrors: true, errorRecovery: true }).diagnostics,
      checked.diagnostics,
    );
  }

  checkRecovery("/* 😀 */ const a; let 文; let 文;", "js");
  checkRecovery("/* 😀 */ const a; type Value = string; type Value = number;", "ts");
});

test("parseSync reports diagnostics with UTF-16 offsets", () => {
  const source = "/* 文😀 */ const = 1";
  const result = parseSync(source, { errorRecovery: true });

  assert.equal(result.diagnostics.length, 1);
  assert.equal(result.diagnostics[0].start, source.indexOf("="));
  assert.equal(result.diagnostics[0].end, source.indexOf("=") + 1);
  assert.equal(result.comments[0].value, " 文😀 ");

  const noComments = parseSync(source, { comments: "none", errorRecovery: true });
  assert.deepEqual(noComments.diagnostics, result.diagnostics);
  assert.deepEqual(noComments.comments, []);

  const eofSource = '"文😀"; const value =';
  const eof = parseSync(eofSource, { errorRecovery: true });
  assert.equal(eof.diagnostics.length, 1);
  assert.equal(eof.diagnostics[0].start, eofSource.length);
  assert.equal(eof.diagnostics[0].end, eofSource.length);
});

test("parseSync preserves diagnostic help, labels, and Unicode messages", () => {
  const source = "/* 😀 */ <开始></结束>";
  const result = parseSync(source, { lang: "jsx", errorRecovery: true });

  assert.equal(result.diagnostics.length, 1);
  const [error] = result.diagnostics;
  assert.equal(error.severity, "error");
  assert.ok(error.message.includes("开始"));
  assert.ok(error.message.includes("结束"));
  assert.ok(error.help.length > 0);
  assert.equal(error.start, source.indexOf("结束"));
  assert.equal(error.end, source.indexOf("结束") + 2);
  assert.equal(error.labels.length, 1);
  assert.ok(error.labels[0].message.length > 0);
  assert.equal(error.labels[0].start, source.indexOf("开始"));
  assert.equal(error.labels[0].end, source.indexOf("开始") + 2);
});

test("parseSync decodes diagnostic arrays and optional metadata", () => {
  const result = parseSync("const a; const b;", { errorRecovery: true });

  assert.equal(result.diagnostics.length, 2);
  assert.equal(result.diagnostics[0].severity, "error");
  assert.equal(result.diagnostics[1].severity, "error");
  assert.ok(result.diagnostics[0].start < result.diagnostics[1].start);

  const ambient = parseSync("declare function f() {}", { lang: "dts", errorRecovery: true });
  assert.equal(ambient.diagnostics.length, 1);
  assert.equal(ambient.diagnostics[0].severity, "error");
  assert.equal(ambient.diagnostics[0].help, undefined);
});

test("parseSync preserves source encoding at the API boundary", () => {
  const empty = parseSync("");
  assert.equal(empty.program.body.length, 0);
  assert.equal(empty.program.end, 0);

  const source = 'const 값 = "한글😀";';
  const result = parseSync(source);
  const standalone = parseStandalone(source);
  assert.equal(typeof Object.getOwnPropertyDescriptor(AstView.prototype, "program").get, "function");
  assert.equal(result.program.end, source.length);
  assert.equal(result.program.body.get(0).declarators.get(0).init.value, "한글😀");
  assert.deepEqual(standalone.materialize(), result.materialize());

  const high = parseSync('"\ud800";');
  assert.equal(high.program.directives.get(0).expression.value, "\ufffd");
  assert.equal(high.program.end, 4);

  const low = parseSync('"\udc00";');
  assert.equal(low.program.directives.get(0).expression.value, "\ufffd");
  assert.equal(low.program.end, 4);

  assert.throws(() => parseSync(undefined), TypeError);
  assert.throws(() => parseSync(123), TypeError);
  assert.throws(() => parseStandalone(null), TypeError);
});

test("path helpers return string enum values through N-API", () => {
  assert.equal(langFromPath("input.js"), "js");
  assert.equal(langFromPath("input.jsx"), "jsx");
  assert.equal(langFromPath("input.ts"), "ts");
  assert.equal(langFromPath("input.tsx"), "tsx");
  assert.equal(langFromPath("input.d.ts"), "dts");
  assert.equal(sourceTypeFromPath("input.cjs"), "commonjs");
  assert.equal(sourceTypeFromPath("input.mjs"), "module");
});

test("path helpers preserve strings across the native boundary", () => {
  assert.equal(langFromPath(""), "js");
  assert.equal(langFromPath("한글😀/input.tsx"), "tsx");
  assert.equal(langFromPath("x\0.ts"), "ts");
  assert.equal(langFromPath("\ud800/input.ts"), "ts");
  assert.equal(sourceTypeFromPath("한글😀/input.cjs"), "commonjs");
  assert.equal(langFromPath("한".repeat(1362) + ".ts"), "ts");
  assert.equal(langFromPath("한글😀/".repeat(1000) + "input.tsx"), "tsx");
  assert.throws(() => langFromPath(123));
  assert.throws(() => sourceTypeFromPath(null));
});
