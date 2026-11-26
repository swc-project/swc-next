import assert from "node:assert/strict";
import test from "node:test";
import { runInNewContext } from "node:vm";
import {
  parseSync,
  parseBufferSync,
  generateSync,
  generateBuffersSync,
  NodeKind,
  readAst,
  walk,
  visitorKeys,
} from "../dist/ffi2.js";

function arrayLayout(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const nodeCount = view.getUint32(0, true);
  const payloads = Math.ceil((8 + nodeCount * 9) / 8) * 8;
  const extraInfo = payloads + nodeCount * 16, extra = extraInfo + 4;
  const stringsInfo = extra + view.getUint32(extraInfo, true) * 4;
  const commentsInfo = Math.ceil((stringsInfo + 8 + view.getUint32(stringsInfo + 4, true)) / 4) * 4;
  const diagnosticsInfo = commentsInfo + 4 + view.getUint32(commentsInfo, true) * 8;
  const positionsInfo = Math.ceil((diagnosticsInfo + 8 + view.getUint32(diagnosticsInfo + 4, true)) / 4) * 4;
  return { view, nodeCount, rootId: view.getUint32(bytes.byteLength - 8, true), payloads, extra, positionsInfo };
}

const typescript = { lang: "ts" };
const jsx = { lang: "jsx" };
const tsx = { lang: "tsx" };

function expectRoundTrip(source, options) {
  const ast = parseSync(source, options);
  const code = generateSync(ast.program).code;
  assert.equal(generateBuffersSync(...ast.encode()).code, code);
  return ast;
}

test("AST buffers own the source and lazily create node views", () => {
  const bytes = parseBufferSync('const value = "你好";');
  const ast = readAst(bytes);
  assert.equal(ast.buffer.buffer, bytes.buffer);
  assert.equal(ast.sourceBytes.buffer, bytes.buffer);
  assert.equal(ast.materializedNodeCount, 0);
  assert.equal(ast.program.type, "Program");
  assert.equal(ast.materializedNodeCount, 1);
  const statement = ast.program.body.get(0);
  assert.ok(statement.isSame(ast.program.body.get(0)));
  assert.equal(Array.isArray(ast.program.body), false);
  const identifier = statement.declarators.get(0).id;
  const again = statement.declarators.get(0).id;
  assert.notEqual(identifier, again);
  assert.ok(identifier.isSame(again));
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

test("child fields read stored IDs without indexing parents", () => {
  const ast = parseSync("call([value]);");
  const value = ast.program.body.get(0).expression.arguments.get(0).elements.get(0);
  assert.equal(value.name, "value");
  assert.equal(ast._arena, null);
  assert.throws(() => ast.node(ast.nodeCount + 1), RangeError);

  const initialized = parseSync("const value = input;");
  const declaration = initialized.program.body.get(0).declarators.get(0);
  const before = initialized.materializedNodeCount;
  const init = initialized.ref(declaration.nodeId, "init");
  assert.equal(initialized.materializedNodeCount, before);
  assert.equal(init, declaration.init.nodeId);
});

test("comment arrays preserve values and canonical offsets", () => {
  const source = "/*雪*/\n// é\nvalue;";
  const ast = parseSync(source);
  const comments = ast.comments.map(({ type, value, start, end }) => ({ type, value, start, end }));
  assert.deepEqual(comments, parseSync(source).comments);
});

test("UTF-16 offsets are computed in JS only when position fields are read", () => {
  const replaced = parseSync('"\ud800";');
  assert.equal(replaced.source, '"\ufffd";');
  assert.equal(replaced.program.directives.get(0).expression.value, "\ufffd");

  const source = '/* ascii */ const value = "text";';
  const ast = readAst(parseBufferSync(source));
  const convert = ast.utf16Offset.bind(ast);
  let conversions = 0;
  ast.utf16Offset = (byte) => {
    conversions++;
    return convert(byte);
  };
  const statement = ast.program.body.get(0);
  assert.equal(statement.type, "VariableDeclaration");
  assert.equal(statement.declarators.get(0).id.name, "value");
  assert.equal(ast.comments[0].value, " ascii ");
  assert.equal(conversions, 0);
  assert.equal(statement.start, source.indexOf("const"));
  assert.equal(ast.program.end, source.length);
  assert.equal(ast.comments[0].end, source.indexOf("*/") + 2);
  assert.ok(conversions > 0);
});

test("non-ASCII source strings preserve values", () => {
  const source = '/* 中😀 */ const 文 = "é";';
  const ast = readAst(parseBufferSync(source));
  const statement = ast.program.body.get(0);
  assert.equal(statement.type, "VariableDeclaration");
  assert.equal(statement.declarators.get(0).id.name, "文");
  assert.equal(ast.comments[0].value, " 中😀 ");
  const byteStart = Buffer.byteLength(source.slice(0, source.indexOf("const")));
  assert.equal(ast.byteStart(statement.nodeId), byteStart);
  assert.equal(statement.start, source.indexOf("const"));
  assert.equal(ast.program.end, source.length);
  assert.equal(ast.comments[0].end, source.indexOf("*/") + 2);
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
  const asciiArrays = arrayLayout(ascii.buffer);
  assert.equal(asciiArrays.view.getUint32(asciiArrays.positionsInfo, true), 0);
  assert.equal(asciiArrays.view.getUint32(asciiArrays.positionsInfo + 4, true), 0);

  const mixed = readAst(parseBufferSync("/*éé中中😀😀xé中😀*/"));
  assert.equal(mixed.utf16Offset(5), 3);
  assert.equal(mixed.utf16Offset(11), 5);
  assert.equal(mixed.sourceUtf16Length, 17);
  assert.equal(mixed.utf8Offset(9), 19);
  assert.equal(mixed.utf8Offset(7), 15);
  assert.equal(mixed.utf8Offset(13), 26);
  assert.equal(mixed.utf8Offset(15), 30);
  assert.equal(mixed.utf16Offset(17), 8);
  assert.equal(mixed.utf16Offset(23), 12);
  assert.equal(mixed.utf16Offset(29), 14);
  assert.equal(mixed.utf8Offset(1), 1);
  const arrays = arrayLayout(mixed.buffer);
  const data = arrays.view, table = arrays.positionsInfo + 8;
  assert.equal(data.getUint32(arrays.positionsInfo, true), 6);
  assert.equal(data.getUint32(arrays.positionsInfo + 4, true), 15);
  const rows = Array.from({ length: 6 }, (_, row) =>
    Array.from({ length: 4 }, (_, field) => data.getUint32(table + row * 16 + field * 4, true)));
  assert.deepEqual(rows, [
    [2, 0, 2, 6], [6, 2, 3, 12], [12, 6, 4, 20],
    [21, 10, 2, 23], [23, 11, 3, 26], [26, 13, 4, 30],
  ]);
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
    view.program.body.get(0).declarators.get(0).init.value,
    "\ud800a\udc00",
  );
  assert.equal(view.source, source);

  const unicode = parseBufferSync("/*é中😀*/");
  const unaligned = new Uint8Array(unicode.length + 1);
  unaligned.set(unicode, 1);
  const indexed = readAst(unaligned.subarray(1));
  assert.equal(indexed.buffer.buffer, unaligned.buffer);
  assert.equal(indexed.utf8Offset(4), 7);
  assert.equal(indexed.utf16Offset(10), 5);
});

test("buffer views use ordinary typed-array and DataView properties across realms", () => {
  const bytes = parseBufferSync("const value = 1;");
  const inputs = runInNewContext(
    "[new Uint8Array(buffer, offset, length), new DataView(buffer, offset, length)]",
    { buffer: bytes.buffer, offset: bytes.byteOffset, length: bytes.byteLength },
  );
  for (const input of inputs) {
    const ast = readAst(input);
    assert.equal(ast.buffer.buffer, bytes.buffer);
    assert.equal(ast.source, "const value = 1;");
    assert.deepEqual(ast.materialize(), parseSync(ast.source).materialize());
  }
});

test("native literal and list fields round-trip through both buffers", () => {
  expectRoundTrip("const value = [1, , 0xFFn, /x/gi, true, null];");
  expectRoundTrip("const text = `a${value}b`;");
  expectRoundTrip("tag`\\unicode`;");
  expectRoundTrip('"use strict"; function f() { "use strict"; return 1; }');
  expectRoundTrip("const {value, ...rest} = source;");
  expectRoundTrip("const [value, ...rest] = source;");
});

test("parameter and declaration fields retain TypeScript ownership", () => {
  expectRoundTrip(
    "function f(this: C, value?: string, other: number = 1, ...rest: string[]) {}",
    typescript,
  );
  expectRoundTrip("const value: number = 1;", typescript);
  expectRoundTrip("try {} catch (error: unknown) {}", typescript);
  expectRoundTrip("type F = <T>(this: C, x: T) => T;", typescript);
  expectRoundTrip(
    "interface C { [key: string]: number; method(x: number): void; }",
    typescript,
  );
  expectRoundTrip(
    "class C { constructor(public value: number) {} }",
    typescript,
  );
});

test("TS method signatures keep their native parent", () => {
  const declarationAst = parseSync("declare function f(): void;", typescript);
  const declaration = declarationAst.program.body.get(0);
  assert.equal(declaration.type, "TsFunction");
  assert.equal(declaration.declare, true);
  assert.equal(declarationAst.cloneNode(declaration).type, "TsFunction");

  const methodAst = parseSync("class C { m(): void; }", typescript);
  const method = methodAst.program.body.get(0).body.body.get(0);
  assert.equal(method.type, "TsMethodDefinition");
  assert.equal(method.value.type, "TsFunction");
  assert.equal(method.value.get("isMethod"), undefined);
  assert.ok(methodAst.cloneNode(method).value.isSame(method.value));
});

test("parameter views expose their pattern and initializer independently", () => {
  const ast = expectRoundTrip("function f(value: T) {}", typescript);
  const fn = ast.program.body.get(0), parameter = fn.params.items.get(0);
  const id = parameter.nodeId;
  assert.equal(ast.kind(id), NodeKind.FormalParameter);
  assert.equal(parameter.initializer, null);
  assert.equal(parameter.typeAnnotation.type, "TsTypeAnnotation");

  const initializer = ast.new_node(NodeKind.NumericLiteral);
  initializer.value = 1;
  parameter.initializer = initializer;
  assert.equal(parameter.nodeId, id);
  assert.equal(parameter.type, "FormalParameter");
  assert.ok(parameter.initializer.isSame(initializer));

  const previous = parameter.pattern;
  const replacement = ast.new_node(NodeKind.BindingIdentifier);
  replacement.name = "next";
  parameter.pattern = replacement;
  assert.equal(previous.name, "value");
  assert.equal(parameter.pattern.name, "next");
});

test("parameter containers expose their own items and rest field", () => {
  const ast = expectRoundTrip("function f(this: T, value: T, ...rest: T[]) {}", typescript);
  const fn = ast.program.body.get(0), container = fn.params;
  assert.equal(fn.thisParam.type, "TsThisParameter");
  assert.equal(container.type, "FormalParameters");
  assert.equal(container.items.length, 1);
  assert.ok(ast.node(container.nodeId).items.get(0).isSame(container.items.get(0)));
  assert.equal(container.rest.type, "FormalParameterRest");
});

test("class, module and JSX nodes round-trip through AST storage", () => {
  expectRoundTrip("@decorate class C { @field accessor value = 1; }");
  expectRoundTrip(
    "abstract class C { abstract method(): void; abstract value: number; }",
    typescript,
  );
  expectRoundTrip("declare function f(value: string): void;", typescript);
  expectRoundTrip('declare module "name";', typescript);
  expectRoundTrip("namespace N { export type T = string; }", typescript);
  expectRoundTrip("declare global { interface C {} }", typescript);
  expectRoundTrip(
    "type T<X> = { readonly [K in keyof X]?: X[K] };",
    typescript,
  );
  expectRoundTrip('const element = <A data-id="a&amp;b">中{x}</A>;', jsx);
  expectRoundTrip("const element = <A<T> value={x as T} />;", tsx);
});

test("native field availability does not depend on the source language", () => {
  const ast = expectRoundTrip("const value = input;");
  const declaration = ast.program.body.get(0).declarators.get(0);
  const binding = declaration.id;
  assert.equal(binding.optional, undefined);
  assert.equal(binding.typeAnnotation, undefined);
  assert.equal(declaration.typeAnnotation, null);
  assert.equal(ast.getField(binding.nodeId, "constructor"), undefined);
  assert.equal(ast.getField(binding.nodeId, "__proto__"), undefined);
  assert.equal(binding.decorators, undefined);

  const typed = expectRoundTrip("function f(value: string) {}", typescript);
  const parameter = typed.program.body.get(0).params.items.get(0);
  assert.deepEqual(parameter.decorators.toArray(), []);
  assert.notEqual(parameter.decorators, parameter.decorators);
  parameter.optional = true;
  parameter.typeAnnotation = null;
  assert.equal(parameter.optional, true);
  assert.equal(parameter.typeAnnotation, null);
});

test("variable and catch annotations stay on their native owners", () => {
  const declaration = parseSync("let value: string;", typescript).program.body.get(0).declarators.get(0);
  assert.equal(declaration.typeAnnotation.typeAnnotation.type, "TsStringKeyword");
  assert.equal(declaration.id.typeAnnotation, undefined);

  const handler = parseSync("try {} catch (error: unknown) {}", typescript).program.body.get(0).handler;
  assert.equal(handler.typeAnnotation.typeAnnotation.type, "TsUnknownKeyword");
  assert.equal(handler.param.typeAnnotation, undefined);
});

test("AST consumers can print and walk views or request independent snapshots", () => {
  const ast = parseSync("function f(value) { return value; }");
  const names = [];
  const record = context => { names.push(context.node.name); };
  walk(ast.program, { BindingIdentifier: record, IdentifierReference: record });
  assert.deepEqual(names, ["f", "value", "value"]);
  const editable = ast.materialize();
  editable.body.at(0).id.name = "renamed";
  assert.equal(ast.program.body.get(0).id.name, "f");
  assert.equal(editable.body.at(0).id.name, "renamed");
  assert.throws(() => generateSync(editable), TypeError);
  ast.program.start = 1;
  assert.equal(ast.program.start, 1);
});

test("parser options and recovered diagnostics cross the AST boundary", () => {
  const options = {
    ...typescript,
    sourceType: "script",
    comments: "none",
    errorRecovery: true,
  };
  const source = "/*中*/ const value: number = ;";
  const ast = parseSync(source, options);
  assert.deepEqual(ast.meta, { lang: "ts", sourceType: "script" });
  assert.deepEqual(ast.comments, []);
  assert.deepEqual(ast.diagnostics, parseSync(source, options).diagnostics);
  assert.throws(() => parseSync(source, typescript));
  const semantic = { semanticErrors: true, errorRecovery: true };
  assert.deepEqual(
    parseSync("let x; let x;", semantic).diagnostics,
    parseSync("let x; let x;", semantic).diagnostics,
  );

  const reentrant = parseSync("outer;", {
    get preserveParens() {
      parseBufferSync("inner;");
      return false;
    },
  });
  assert.equal(reentrant.source, "outer;");
  assert.equal(reentrant.program.body.get(0).expression.name, "outer");
});

test("walk contexts isolate nested calls and release state after return or error", () => {
  const outer = parseSync("outer();").program;
  const inner = parseSync("inner();").program;
  const state = { name: "outer" };
  let retained;
  walk(outer, {
    CallExpression(context) {
      const node = context.node;
      retained = context;
      const ancestors = context.ancestors();
      walk(inner, {
        IdentifierReference(nested) {
          assert.notEqual(nested, context);
          assert.equal(nested.state, "inner");
        },
      }, "inner");
      assert.equal(context.state, state);
      assert.ok(context.node.isSame(node));
      assert.deepEqual(context.ancestors(), ancestors);
    },
  }, state);
  assert.throws(() => retained.node);
  assert.throws(() => walk(inner, { enter() { throw new Error("hook failed"); } }));
  const seen = [];
  walk(outer, {
    enter(context) {
      assert.equal(context.state, "next");
      seen.push(context.type);
      context.stop();
    },
  }, "next");
  assert.deepEqual(seen, ["Program"]);
});

test("invalid buffers are rejected before allocation or during bounded access", () => {
  assert.throws(() => readAst(new Uint8Array(0)), RangeError);
  assert.throws(() => parseBufferSync(123), TypeError);
  const truncated = parseBufferSync("value;").slice(0, 4);
  assert.throws(() => readAst(truncated), RangeError);

  const counts = parseBufferSync("value;").slice();
  new DataView(counts.buffer, counts.byteOffset, counts.byteLength).setUint32(0, 0xffffffff, true);
  assert.throws(() => readAst(counts), RangeError);

  const positions = parseBufferSync("/*é*/").slice();
  const positionArrays = arrayLayout(positions);
  positionArrays.view.setUint32(positionArrays.positionsInfo, 0xffffffff, true);
  assert.throws(() => readAst(positions), RangeError);

  const invalidRun = parseBufferSync("/*é*/").slice();
  const runArrays = arrayLayout(invalidRun);
  runArrays.view.setUint32(runArrays.positionsInfo + 16, 0, true);
  const malformed = readAst(invalidRun);
  assert.throws(() => malformed.utf16Offset(3), RangeError);
  assert.throws(() => malformed.utf8Offset(3), RangeError);

  const cyclic = parseBufferSync("value;").slice();
  const cyclicArrays = arrayLayout(cyclic);
  cyclicArrays.view.setUint32(cyclicArrays.payloads + 16, 1, true);
  assert.throws(() => generateBuffersSync(cyclic));

  const listBytes = parseBufferSync("x()").slice();
  const listArrays = arrayLayout(listBytes);
  const extraCount = listArrays.view.getUint32(listArrays.extra - 4, true);
  listArrays.view.setUint32(listArrays.payloads + listArrays.rootId * 16 + 12, extraCount - 1, true);
  listArrays.view.setUint32(listArrays.extra + (extraCount - 1) * 4, 0, true);
  const listAst = readAst(listBytes);
  assert.throws(() => listAst.program.body.length, RangeError);
  assert.throws(() => listAst.program.get("body").length, RangeError);
  assert.throws(() => generateBuffersSync(listAst.buffer));

  const templateAst = parseSync("`x`");
  const template = templateAst.program.body.get(0).expression.quasis.get(0);
  const templateArrays = arrayLayout(templateAst.buffer);
  const templateExtraCount = templateArrays.view.getUint32(templateArrays.extra - 4, true);
  templateArrays.view.setUint32(templateArrays.payloads + template.nodeId * 16 + 12, templateExtraCount - 1, true);
  templateArrays.view.setUint32(templateArrays.extra + (templateExtraCount - 1) * 4, 0, true);
  assert.throws(() => template.get("raw"), RangeError);
  assert.throws(() => templateAst.bytesForString(template.nodeId, "raw"), RangeError);
});

test("deep AST access and materialization do not recurse through the whole tree", () => {
  const source = `type Value = string${"[]".repeat(600)};`;
  const ast = parseSync(source, typescript);
  let type = ast.program.body.get(0).typeAnnotation;
  for (let i = 0; i < 600; i++) { type = type.elementType; }
  assert.equal(type.type, "TsStringKeyword");
  assert.deepEqual(ast.materialize(), parseSync(source, typescript).materialize());
});

test("buffer walkers preserve visitor order without materializing public lists", () => {
  const ast = parseSync("fn(1, 2);");
  const plain = parseSync("fn(1, 2);").program;
  function trace(program) {
    const entries = [];
    walk(program, {
      enter(context) {
        entries.push([
          "enter",
          context.type,
          context.parent?.type,
          context.key,
          context.index,
        ]);
      },
      leave(context) {
        entries.push([
          "leave",
          context.type,
          context.parent?.type,
          context.key,
          context.index,
        ]);
      },
    });
    return entries;
  }
  assert.deepEqual(trace(ast.program), trace(plain));
  assert.equal(ast.encode()[1].length, 0);
});

test("parent relationships belong to the active walker context", () => {
  const ast = parseSync("function f(value: T = 1) {}", typescript);
  const parameter = ast.program.body.get(0).params.items.get(0);
  assert.equal("parent" in parameter, false);
  assert.equal("parentId" in ast, false);
  assert.equal("_parents" in ast, false);
  assert.equal("_currentParents" in ast, false);
  const relationships = program => {
    const result = [];
    walk(program, { enter(context) {
      result.push([context.type, context.parent?.type, context.key]);
    } });
    return result;
  };
  const relationshipsPlain = root => {
    const result = [];
    const visit = (node, parent, key) => {
      result.push([node.type, parent?.type, key]);
      for (const childKey of visitorKeys[node.type] ?? []) {
        const value = node[childKey];
        if (Array.isArray(value)) {
          for (const child of value) {
            if (child != null) visit(child, node, childKey);
          }
        } else if (value != null && typeof value.type === "string") {
          visit(value, node, childKey);
        }
      }
    };
    visit(root, null, null);
    return result;
  };
  assert.deepEqual(relationships(ast.program), relationshipsPlain(ast.materialize()));
  parameter.pattern.name = "renamed";
  assert.equal(parameter.pattern.name, "renamed");
  assert.equal("traverse" in ast, false);
  assert.equal("firstChildId" in ast, false);
  assert.equal("nextSiblingId" in ast, false);
  assert.equal("_navigationChildren" in ast, false);
});
