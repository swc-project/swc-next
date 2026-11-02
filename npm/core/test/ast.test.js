import assert from "node:assert/strict";
import test from "node:test";
import { runInNewContext } from "node:vm";
import {
  parseSync,
  parseAstSync,
  parseBufferSync,
  printSync,
  NodeKind,
  NodeView,
  readAst,
  walk,
} from "../src-js/index.js";

const typescript = { lang: "ts" };
const jsx = { lang: "jsx" };
const tsx = { lang: "tsx" };

function expectProjection(source, options) {
  const ast = parseAstSync(source, options);
  const expected = parseSync(source, options);
  assert.deepEqual(ast.materialize(), expected.program);
  assert.deepEqual(ast.meta, expected.meta);
  assert.equal(
    printSync(ast.program, {}, ast.meta).code,
    printSync(expected.program, {}, expected.meta).code,
  );
  return ast;
}

test("FFI2 buffers own the source and lazily create node views", () => {
  const bytes = parseBufferSync('const value = "你好";');
  const ast = readAst(bytes);
  assert.equal(ast.buffer.buffer, bytes.buffer);
  assert.equal(ast.sourceBytes.buffer, bytes.buffer);
  assert.equal(ast.materializedNodeCount, 0);
  const seen = [];
  ast.traverse((id, kind) => {
    seen.push([id, kind]);
  });
  assert.equal(seen[0][0], ast.rootId);
  assert.equal(new Set(seen.map(([id]) => id)).size, seen.length);
  assert.equal(ast.materializedNodeCount, 0);
  assert.equal(ast.program.type, "Program");
  assert.equal(ast.materializedNodeCount, 1);
  const statement = ast.program.body[0];
  assert.equal(statement, ast.program.body[0]);
  assert.ok(Array.isArray(ast.program.body));
  assert.ok(NodeView.isSame(statement.parent, ast.program));
  const identifier = statement.declarations[0].id;
  const again = statement.declarations[0].id;
  assert.notEqual(identifier, again);
  assert.ok(NodeView.isSame(identifier, again));
  again.name = "renamed";
  assert.equal(identifier.name, "renamed");
  identifier.name = "value";
  assert.equal(identifier.name, "value");
  assert.equal(
    ast.bytesForString(identifier.nodeId, "name").buffer,
    ast.editBuffer.buffer,
  );
  assert.equal(ast.source, 'const value = "你好";');
});

test("UTF-16 offsets are computed in JS only when position fields are read", () => {
  const source = '/* 中😀 */ const 文 = "é";';
  const ast = readAst(parseBufferSync(source));
  const convert = ast.utf16Offset.bind(ast);
  let conversions = 0;
  ast.utf16Offset = (byte) => {
    conversions++;
    return convert(byte);
  };
  const statement = ast.program.body[0];
  assert.equal(statement.type, "VariableDeclaration");
  assert.equal(statement.declarations[0].id.name, "文");
  assert.equal(ast.comments[0].value, " 中😀 ");
  assert.equal(conversions, 0);
  const byteStart = Buffer.byteLength(source.slice(0, source.indexOf("const")));
  assert.equal(ast.byteStart(statement.nodeId), byteStart);
  assert.equal(statement.start, source.indexOf("const"));
  assert.equal(ast.program.end, source.length);
  assert.equal(ast.comments[0].end, source.indexOf("*/") + 2);
  assert.ok(conversions > 0);
});

test("UTF-8 position indexing supports out-of-order access and surrogate pairs", () => {
  const source = "a中😀éz";
  const ast = readAst(parseBufferSync(`/*${source}*/`));
  assert.equal(ast.utf16Offset(14), 9);
  assert.equal(ast.utf16Offset(3), 3);
  assert.equal(ast.utf16Offset(6), 4);
  assert.equal(ast.utf16Offset(10), 6);
  assert.equal(ast.utf16Offset(11), 7);
  assert.equal(ast.utf16Offset(13), 8);
  assert.equal(ast.utf16Offset(0), 0);
  assert.equal(ast.sourceUtf16Length, source.length + 4);
  assert.equal(ast.utf8Offset(9), 14);
  assert.equal(ast.utf8Offset(3), 3);
  assert.equal(ast.utf8Offset(5), 9);
  assert.equal(ast.utf8Offset(6), 10);
  assert.equal(ast.utf8Offset(4), 6);
  assert.equal(ast.utf8Offset(11), 16);
  assert.equal(ast.utf8Offset(2), 2);
  assert.equal(ast.utf8Offset(8), 13);

  const large = `/*${"a".repeat(255)}中${"b".repeat(255)}😀*/`;
  const indexed = readAst(parseBufferSync(large));
  assert.equal(indexed.program.end, large.length);
  assert.equal(
    indexed.utf16Offset(Buffer.byteLength(large.slice(0, 258))),
    258,
  );
  assert.equal(indexed.utf16Offset(257), 257);
  assert.equal(indexed.utf8Offset(large.length), Buffer.byteLength(large));
  assert.equal(indexed.utf8Offset(258), 260);
  assert.equal(indexed.utf8Offset(259), 261);
  assert.equal(indexed.utf8Offset(514), 518);
  assert.equal(indexed.utf8Offset(515), 519);
  assert.equal(indexed.utf8Offset(257), 257);

  const ascii = readAst(parseBufferSync(`/*${"a".repeat(512)}*/`));
  assert.equal(ascii.utf8Offset(257), 257);
  assert.equal(ascii.utf8Offset(1), 1);
  assert.throws(() => ascii.utf8Offset(517), RangeError);
});

test("buffer views support unaligned offsets and lazy WTF-8 strings", () => {
  const source = 'const value = "\\uD800\\u0061\\uDC00";';
  const bytes = parseBufferSync(source);
  const padded = new Uint8Array(bytes.length + 3);
  padded.set(bytes, 1);
  const view = readAst(padded.subarray(1, bytes.length + 1));
  assert.equal(view.buffer.buffer, padded.buffer);
  assert.equal(view.buffer.byteOffset, 1);
  assert.equal(
    view.program.body[0].declarations[0].init.value,
    "\ud800a\udc00",
  );
  assert.equal(view.source, source);
});

test("buffer views read intrinsic storage across realms", () => {
  const bytes = parseBufferSync("const value = 1;");
  const inputs = runInNewContext(
    "[new Uint8Array(buffer, offset, length), new DataView(buffer, offset, length)]",
    { buffer: bytes.buffer, offset: bytes.byteOffset, length: bytes.byteLength },
  );
  for (const input of inputs) {
    const ast = readAst(input);
    assert.equal(ast.buffer.buffer, bytes.buffer);
    assert.equal(ast.source, "const value = 1;");
    assert.deepEqual(ast.materialize(), parseSync(ast.source).program);
  }
});

test("literal fields and list fields match the canonical ESTree API", () => {
  expectProjection("const value = [1, , 0xFFn, /x/gi, true, null];");
  expectProjection("const text = `a${value}b`;");
  expectProjection("tag`\\unicode`;");
  expectProjection('"use strict"; function f() { "use strict"; return 1; }');
  expectProjection("const {value, ...rest} = source;");
  expectProjection("const [value, ...rest] = source;");
});

test("parameter and declaration projections preserve TypeScript ownership", () => {
  expectProjection(
    "function f(this: C, value?: string, other: number = 1, ...rest: string[]) {}",
    typescript,
  );
  const declaration = expectProjection("const value: number = 1;", typescript)
    .program.body[0].declarations[0];
  assert.ok(
    NodeView.isSame(declaration.id.typeAnnotation.parent, declaration.id),
  );
  expectProjection("try {} catch (error: unknown) {}", typescript);
  expectProjection("type F = <T>(this: C, x: T) => T;", typescript);
  expectProjection(
    "interface C { [key: string]: number; method(x: number): void; }",
    typescript,
  );
  expectProjection(
    "class C { constructor(public value: number) {} }",
    typescript,
  );
});

test("class, module and JSX projections match the existing decoder", () => {
  expectProjection("@decorate class C { @field accessor value = 1; }");
  expectProjection(
    "abstract class C { abstract method(): void; abstract value: number; }",
    typescript,
  );
  expectProjection("declare function f(value: string): void;", typescript);
  expectProjection('declare module "name";', typescript);
  expectProjection("namespace N { export type T = string; }", typescript);
  expectProjection("declare global { interface C {} }", typescript);
  expectProjection(
    "type T<X> = { readonly [K in keyof X]?: X[K] };",
    typescript,
  );
  expectProjection('const element = <A data-id="a&amp;b">中{x}</A>;', jsx);
  expectProjection("const element = <A<T> value={x as T} />;", tsx);
});

test("projected fields preserve language visibility, defaults and edits", () => {
  const javascript = expectProjection("const value = input;");
  const jsId = javascript.program.body[0].declarations[0].id;
  assert.equal(jsId.optional, undefined);
  assert.equal(jsId.typeAnnotation, undefined);
  assert.equal(javascript.getField(jsId.nodeId, "constructor"), undefined);
  assert.equal(javascript.getField(jsId.nodeId, "__proto__"), undefined);

  const typed = expectProjection("function f(value: string) {}", typescript);
  const tsId = typed.program.body[0].params[0];
  assert.equal(tsId.optional, false);
  assert.equal(tsId.typeAnnotation.type, "TSTypeAnnotation");
  assert.deepEqual(tsId.decorators, []);
  assert.equal(tsId.decorators, tsId.decorators);
  tsId.optional = true;
  tsId.typeAnnotation = null;
  assert.equal(tsId.optional, true);
  assert.equal(tsId.typeAnnotation, null);

  const program = typed.program;
  program.body = [];
  assert.deepEqual(typed.materialize().body, []);
  assert.equal(javascript.program.body.length, 1);
});

test("canonical AST consumers can print and walk views or request editable objects", () => {
  const ast = parseAstSync("function f(value) { return value; }");
  const names = [];
  walk(ast.program, {
    Identifier(node) {
      names.push(node.name);
    },
  });
  assert.deepEqual(names, ["f", "value", "value"]);
  const editable = ast.materialize();
  editable.body[0].id.name = "renamed";
  assert.equal(ast.program.body[0].id.name, "f");
  assert.match(printSync(editable, {}, ast.meta).code, /function renamed/);
  ast.program.start = 1;
  assert.equal(ast.program.start, 1);
});

test("parser options and recovered diagnostics cross the new FFI boundary", () => {
  const options = {
    ...typescript,
    sourceType: "script",
    comments: "none",
    errorRecovery: true,
  };
  const source = "/*中*/ const value: number = ;";
  const ast = parseAstSync(source, options);
  assert.deepEqual(ast.meta, { lang: "ts", sourceType: "script" });
  assert.deepEqual(ast.comments, []);
  assert.deepEqual(ast.diagnostics, parseSync(source, options).diagnostics);
  assert.throws(() => parseAstSync(source, typescript));
  const semantic = { semanticErrors: true, errorRecovery: true };
  assert.deepEqual(
    parseAstSync("let x; let x;", semantic).diagnostics,
    parseSync("let x; let x;", semantic).diagnostics,
  );
});

test("native traversal can skip subtrees and stop without reading ESTree fields", () => {
  const ast = parseAstSync("function f() { return 1; } next();");
  const skipped = [];
  ast.traverse((id, kind) => {
    skipped.push(kind);
    if (kind === NodeKind.Function) return "skip";
  });
  assert.ok(!skipped.includes(NodeKind.ReturnStatement));
  let count = 0;
  ast.traverse(() => {
    count++;
    return "stop";
  });
  assert.equal(count, 1);
  assert.equal(ast.materializedNodeCount, 0);
});

test("native navigation can be requested after ordinary traversal and edits", () => {
  const ast = parseAstSync("const [first, ...rest] = input;", typescript);
  const original = readAst(ast.buffer);
  const links = (view) => {
    const result = [];
    view.traverse((id, kind) => {
      result.push([id, kind, view.parentId(id), view.firstChildId(id), view.nextSiblingId(id)]);
    });
    return result;
  };
  const expected = links(original);
  walk(ast.program, {});
  assert.notEqual(ast._parents, null);
  assert.equal(ast._firstChildren, null);
  assert.equal(ast._nextSiblings, null);
  ast.program.body.splice(0, 1);
  assert.deepEqual(links(ast), expected);
  assert.deepEqual(ast.materialize().body, []);
  assert.deepEqual(links(ast), expected);
});

test("walk contexts isolate nested calls and release state after return or error", () => {
  const outer = parseAstSync("outer();").program;
  const inner = parseAstSync("inner();").program;
  const state = { name: "outer" };
  let retained;
  walk(outer, {
    CallExpression(node, context) {
      retained = context;
      const ancestors = context.ancestors();
      walk(inner, {
        Identifier(_node, nested) {
          assert.notEqual(nested, context);
          assert.equal(nested.state, "inner");
        },
      }, "inner");
      assert.equal(context.state, state);
      assert.ok(NodeView.isSame(context.node, node));
      assert.deepEqual(context.ancestors(), ancestors);
    },
  }, state);
  assert.throws(() => retained.node);
  assert.throws(() => walk(inner, { enter() { throw new Error("hook failed"); } }));
  const seen = [];
  walk(outer, {
    enter(node, context) {
      assert.equal(context.state, "next");
      seen.push(node.type);
      context.stop();
    },
  }, "next");
  assert.deepEqual(seen, ["Program"]);
});

test("invalid buffers are rejected before allocation or during bounded access", () => {
  assert.throws(() => readAst(new Uint8Array(0)), RangeError);
  assert.throws(() => parseBufferSync(123), TypeError);
  const version = parseBufferSync("value;").slice();
  new DataView(
    version.buffer,
    version.byteOffset,
    version.byteLength,
  ).setUint32(4, 999, true);
  assert.throws(() => readAst(version), RangeError);
  const counts = parseBufferSync("value;").slice();
  new DataView(counts.buffer, counts.byteOffset, counts.byteLength).setUint32(
    20,
    0xffffffff,
    true,
  );
  assert.throws(() => readAst(counts), RangeError);
  const cyclic = parseBufferSync("value;").slice();
  const view = new DataView(
    cyclic.buffer,
    cyclic.byteOffset,
    cyclic.byteLength,
  );
  const payloads =
    Math.ceil((view.getUint32(28, true) + view.getUint32(20, true) * 9) / 8) *
    8;
  view.setUint32(payloads + 16, 1, true);
  assert.throws(() => readAst(cyclic).traverse(() => {}), RangeError);
});

test("deep AST access and materialization do not recurse through the whole tree", () => {
  const source = `type Value = string${"[]".repeat(600)};`;
  const ast = parseAstSync(source, typescript);
  let type = ast.program.body[0].typeAnnotation;
  for (let i = 0; i < 600; i++) type = type.elementType;
  assert.equal(type.type, "TSStringKeyword");
  assert.deepEqual(ast.materialize(), parseSync(source, typescript).program);
});

test("buffer walkers preserve visitor order without materializing public lists", () => {
  const ast = parseAstSync("fn(1, 2);");
  const plain = parseSync("fn(1, 2);").program;
  function trace(program) {
    const entries = [];
    walk(program, {
      enter(node, context) {
        entries.push([
          "enter",
          node.type,
          context.parent?.type,
          context.key,
          context.index,
        ]);
      },
      leave(node, context) {
        entries.push([
          "leave",
          node.type,
          context.parent?.type,
          context.key,
          context.index,
        ]);
      },
    });
    return entries;
  }
  assert.deepEqual(trace(ast.program), trace(plain));
  assert.equal(ast._watchedLists.size, 0);
  assert.equal(ast.encode()[1].length, 0);
});
