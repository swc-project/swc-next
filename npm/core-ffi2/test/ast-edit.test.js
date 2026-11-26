import assert from "node:assert/strict";
import test from "node:test";
import {
  parseSync,
  parseBufferSync,
  readAst,
  AstList,
  NodeKind,
  generateSync,
  generateBuffersSync,
  walk,
} from "@swc-next/core-ffi2";

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
function identifier(ast, name, kind = NodeKind.IdentifierReference) {
  const node = ast.new_node(kind);
  node.name = name;
  return node;
}
function literal(ast, value) {
  const node = ast.new_node(typeof value === "number" ? NodeKind.NumericLiteral : NodeKind.StringLiteral);
  node.value = value;
  return node;
}
function expression(value) {
  const node = value.ast.new_node(NodeKind.ExpressionStatement);
  node.set("expression", value);
  return node;
}

function transform(source, change, options = {}) {
  const ast = parseSync(source, options);
  const snapshot = ast.buffer.slice();
  change(ast.program, ast);
  const [buffer, extension] = ast.encode();
  assert.equal(buffer, ast.buffer);
  assert.equal(buffer.byteLength, snapshot.byteLength);
  const result = generateBuffersSync(buffer, extension);
  assert.equal(generateSync(ast.program).code, result.code);
  return { ast, extension, code: result.code };
}

function expectCode(actual, source, options) {
  const { program } = parseSync(source, options);
  assert.equal(actual, generateSync(program).code);
}

function viewedNodes(ast, node = ast.program) {
  const nodes = [];
  walk(node, { enter(value) { nodes.push([value.nodeId, ast.kind(value.nodeId)]); } });
  return nodes;
}

test("native string fields write and read the same storage", () => {
  const directiveAst = parseSync('"use strict";');
  const directive = directiveAst.program.directives.get(0);
  directive.value = "use asm";
  assert.equal(directive.value, "use asm");
  assert.equal(directive.get("value"), "use asm");
  assert.equal(directiveAst.materialize().directives[0].value, "use asm");

  const textAst = parseSync("<div>hello</div>;", jsx);
  const text = textAst.program.body.get(0).expression.children.get(0);
  text.value = "world";
  assert.equal(text.value, "world");
  assert.equal(textAst.cloneNode(text).value, "world");
  expectCode(generateSync(textAst.program).code, "<div>world</div>;", jsx);

  text.value = "again";
  assert.equal(text.value, "again");
  assert.equal(textAst.materialize().body[0].expression.children[0].value, "again");

  const surrogates = parseSync('["\\uD800", "\\uDC00"];');
  const elements = surrogates.program.body.get(0).expression.elements;
  assert.equal(elements.get(0).value, "\ud800");
  assert.equal(elements.get(1).value, "\udc00");
  expectCode(generateBuffersSync(...surrogates.encode()).code, '["\\uD800", "\\uDC00"];');
});

test("native spans and parameter lists retain their owning nodes", () => {
  const programAst = parseSync("  value;", typescript);
  const statementStart = programAst.program.body.get(0).start;
  programAst.program.start = 1;
  assert.equal(programAst.program.start, 1);
  assert.equal(programAst.program.body.get(0).start, statementStart);

  const decoratedAst = parseSync("@dec class C {}", typescript);
  decoratedAst.program.start = 1;
  assert.equal(decoratedAst.program.start, 1);

  const parameterAst = parseSync("function f(value: string) {}", typescript);
  const fn = parameterAst.program.body.get(0), parameter = fn.params.items.get(0);
  assert.equal(fn.params.items.nodeId(0), parameter.nodeId);
  const end = parameter.end - 1;
  parameter.end = end;
  assert.equal(parameter.end, end);
  assert.equal(fn.get("params").get("items").get(0).end, end);

  const annotationAst = parseSync("let value: string;", typescript);
  const declaration = annotationAst.program.body.get(0).declarators.get(0);
  const binding = declaration.id;
  const annotationEnd = declaration.typeAnnotation.end;
  const bindingEnd = binding.end - 1;
  binding.end = bindingEnd;
  assert.equal(binding.end, bindingEnd);
  assert.equal(declaration.typeAnnotation.end, annotationEnd);

  const decoratorAst = parseSync("class C { method(@dec value: string) {} }", typescript);
  const pattern = decoratorAst.program.body.get(0).body.body.get(0).value.params.items.get(0);
  const decorator = pattern.decorators.get(0);
  pattern.decorators = [];
  assert.equal(pattern.decorators.length, 0);
  pattern.decorators.push(decorator);
  assert.ok(pattern.decorators.get(0).isSame(decorator));
  assert.equal(decoratorAst.materialize().body[0].body.body[0].value.params.items[0].decorators.length, 1);
});

test("list growth reuses capacity and updates live references", () => {
  const ast = parseSync("[];");
  const array = ast.program.body.get(0).expression, list = array.elements;
  for (let index = 0; index < 1024; index++) { list.push(literal(ast, 1)); }
  const value = list.get(0), last = list.get(1023), other = literal(ast, 2);
  assert.equal(list.length, 1024);
  assert.ok(ast._arena.extraUsed < 4096);
  const used = ast._arena.extraUsed;
  list.set(0, other);
  assert.equal(ast._arena.extraUsed, used);
  assert.equal(list.get(0).value, 2);
  assert.equal(value.value, 1);

  list.splice(1, 1022);
  const appended = literal(ast, 3);
  list.push(appended);
  assert.equal(ast._arena.extraUsed, used);
  assert.deepEqual(list.map(node => node.value), [2, 1, 3]);
  list.set(1, literal(ast, 4));
  assert.equal(last.value, 1);
  expectCode(generateBuffersSync(...ast.encode()).code, "[2, 4, 3];");
  list.clear();
});

test("splicing an elision returns its null entry", () => {
  const ast = parseSync("[,];");
  const array = ast.program.body.get(0).expression;
  const removed = ast.spliceList(array, "elements", 0, 1);
  assert.deepEqual(removed, [null]);
  expectCode(generateBuffersSync(...ast.encode()).code, "[];");
});

test("metadata overrides remain the default for later encoding", () => {
  const ast = parseSync("value;", { sourceType: "module" });
  const first = ast.encode({ sourceType: "script" });
  const flags = first[0].slice(-4);
  ast.encode();
  assert.deepEqual(first[0].slice(-4), flags);
  assert.equal(ast.meta.sourceType, "script");
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "value;");
});

test("detached appended nodes expose their current child fields", () => {
  const ast = parseSync("original;");
  const block = ast.new_node(NodeKind.BlockStatement);
  const statement = expression(literal(ast, 2));
  block.body.push(statement);
  assert.ok(block.body.get(0).isSame(statement));
  ast.program.body = [block];
  assert.ok(ast.program.body.get(0).isSame(block));
  expectCode(generateSync(ast.program).code, "{ 2; }");
});

test("unchanged views use the original buffer with an empty extension", () => {
  const ast = parseSync('const 文 = "你好";');
  const [buffer, extension] = ast.encode();
  assert.equal(buffer, ast.buffer);
  assert.equal(extension.length, 0);
  assert.equal(ast.materializedNodeCount, 0);
  expectCode(generateBuffersSync(buffer, extension).code, 'const 文 = "你好";');
  ast.materialize();
  assert.equal(ast.encode()[1].length, 0);
});

test("setters update original fields without appending node rows", () => {
  const result = transform(
    "let x = 1; function keep() { return longCall(a, b, c); }",
    (program, ast) => {
      program.body.get(0).declarators.get(0).id.name = "renamed";
    },
  );
  expectCode(
    result.code,
    "let renamed = 1; function keep() { return longCall(a, b, c); }",
  );
  const view = new DataView(result.extension.buffer);
  assert.equal(view.getUint32(0, true), 0);
  assert.equal(view.getUint32(result.extension.byteLength - 20, true), 0);
});

test("original scalar and list storage updates without appended data", () => {
  const scalar = parseSync("const value = 1;");
  const number = scalar.program.body.get(0).declarators.get(0).init;
  const bytes = scalar.buffer, count = scalar.totalNodeCount;
  const payload = arrayLayout(bytes).payloads + number.nodeId * 16;
  number.value = 2;
  assert.equal(new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getFloat64(payload,true),2);
  assert.equal(scalar.buffer,bytes);
  assert.equal(scalar.totalNodeCount,count);
  assert.equal(scalar.encode()[1].length,0);
  expectCode(generateSync(scalar.program).code,"const value = 2;");

  const list = parseSync("[one, two];");
  const array = list.program.body.get(0).expression;
  const first = array.elements.get(0), second = array.elements.get(1);
  array.elements = [second];
  assert.equal(list.encode()[1].length,0);
  expectCode(generateSync(list.program).code,"[two];");
});

test("node creation publishes appended bytes and original fields remain mutable", () => {
  const ast = parseSync("original;");
  const base = ast.buffer.slice();
  const value = literal(ast, 42);
  assert.equal("_edits" in ast, false);
  assert.equal(value.value, 42);
  assert.equal(new DataView(ast.editBuffer.buffer).getUint32(ast.editBuffer.byteLength - 20, true), 1);
  const statement = ast.program.body.get(0);
  const statementId = statement.nodeId;
  const previous = statement.expression;
  statement.expression = value;
  assert.equal(ast.program.body.get(0).nodeId, statementId);
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "42;");
  assert.equal(ast.buffer.byteLength, base.byteLength);

  const rows = ast._arena.rowCount;
  const first = ast.encode().map(bytes => bytes.slice());
  assert.equal(ast._arena.rowCount, rows);
  const frozen = first[1].slice();
  value.value = 100.5;
  assert.equal("_edits" in ast, false);
  assert.equal(ast._arena.rowCount, rows);
  assert.deepEqual(first[1], frozen);
  expectCode(generateBuffersSync(...first).code, "42;");
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "100.5;");
  assert.equal(ast.buffer.byteLength, base.byteLength);
});



test("node references clone foreign buffers and reject ordinary objects", () => {
  const ast = parseSync("original;");
  const foreign = parseSync("other;");
  const statement = ast.program.body.get(0);
  const before = ast.buffer.slice();
  assert.throws(() => { statement.expression = { type: "IdentifierReference", name: "plain" }; }, TypeError);
  assert.throws(() => { ast.program.body.push({ type: "EmptyStatement" }); }, TypeError);
  assert.deepEqual(ast.buffer, before);
  expectCode(generateSync(ast.program).code, "original;");

  statement.expression = foreign.program.body.get(0).expression;
  assert.notEqual(statement.expression.nodeId, foreign.program.body.get(0).expression.nodeId);
  expectCode(generateSync(ast.program).code, "other;");
  expectCode(generateSync(foreign.program).code, "other;");
});

test("buffer lists support insertion, removal, replacement and ordering", () => {
  const inserted = transform("a(); b();", (program, ast) => {
    program.body.splice(1, 0, expression(literal(ast, 3)));
  });
  expectCode(inserted.code, "a(); 3; b();");

  const removed = transform("a(); b();", (program, ast) => {
    program.body.splice(0, 1);
  });
  expectCode(removed.code, "b();");

  const replaced = transform("a();", (program, ast) => {
    program.body.set(0, expression(identifier(ast, "replacement")));
  });
  expectCode(replaced.code, "replacement;");

  const reordered = transform("a(); b();", (program, ast) => {
    program.body.reverse();
  });
  expectCode(reordered.code, "b(); a();");
});

test("list replacements preserve storage for unchanged references and clone foreign ones", () => {
  const emptyAst = parseSync("<div/>;", jsx);
  const attributes = emptyAst.program.body.get(0).expression.openingElement.attributes;
  attributes.replace([]);
  assert.equal(emptyAst._editVersion, 0);
  assert.equal(emptyAst._listVersion, 0);
  assert.equal(emptyAst._arena, null);

  const ast = parseSync("[one, two];");
  const elements = ast.program.body.get(0).expression.elements;
  const original = ast.buffer.slice();
  const iterator = elements[Symbol.iterator]();
  assert.equal(iterator.next().value.name, "one");
  elements.replace(elements.toArray());
  assert.equal(ast._editVersion, 0);
  assert.equal(ast._listVersion, 0);
  assert.equal(ast._arena, null);
  assert.deepEqual(ast.buffer, original);
  assert.equal(iterator.next().value.name, "two");
  assert.equal(iterator.next().done, true);

  const foreign = parseSync("[one, two];").program.body.get(0).expression.elements;
  elements.replace(foreign);
  assert.equal(elements.get(0).name, "one");
  assert.equal(elements.get(1).name, "two");
  assert.ok(ast._listVersion > 0);
  expectCode(generateSync(ast.program).code, "[one, two];");

  const version = ast._listVersion;
  const replacement = identifier(ast, "changed");
  elements.replace([replacement, elements.get(1)]);
  assert.equal(elements.get(0).name, "changed");
  assert.ok(ast._listVersion > version);
  expectCode(generateSync(ast.program).code, "[changed, two];");
});

test("buffer list views publish mutations before encoding", () => {
  const ast = parseSync("[one, two];");
  const array = ast.program.body.get(0).expression;
  const before = ast.materializedNodeCount;
  const elements = array.elements;
  assert.ok(elements instanceof AstList);
  assert.equal(Array.isArray(elements), false);
  assert.equal(ast.materializedNodeCount, before);
  assert.equal(elements.length, 2);

  const first = elements.get(0), second = elements.get(1);
  elements.pop();
  elements.set(0, second);
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer ?? new Uint8Array()).code, "[two];");
  assert.equal(ast.editBuffer, null);

  elements.push(literal(ast, 3));
  assert.equal(elements.length, 2);
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "[two, 3];");
  assert.equal(elements.get(1).value, 3);
  assert.equal(ast.buffer.byteLength, array.ast.buffer.byteLength);

  const count = ast.totalNodeCount;
  elements.clear();
  assert.equal(elements.length, 0);
  assert.equal(ast.totalNodeCount, count);
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "[];");
  assert.throws(() => { elements[0] = first; }, TypeError);
});

test("new native slots use ordinary field setters", () => {
  const ast = parseSync("value;");
  const base = ast.buffer, before = ast.totalNodeCount;
  const created = ast.new_node(NodeKind.IdentifierReference);
  created.name = "created";
  const statement = ast.program.body.get(0);
  statement.expression = created;
  assert.equal(ast.totalNodeCount, before + 1);
  assert.equal(ast.buffer, base);
  assert.ok(statement.expression.isSame(created));
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "created;");

  created.name = "updated";
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "updated;");
});

test("native construction preserves parameter containers and live native ranges", () => {
  const ast = parseSync("");
  const fn = ast.new_node(NodeKind.Function);
  fn.set("functionType", "FunctionDeclaration");
  fn.set("id", identifier(ast, "created", NodeKind.BindingIdentifier));
  fn.set("body", ast.new_node(NodeKind.FunctionBody));
  const parameter = ast.new_node(NodeKind.FormalParameter);
  parameter.pattern = identifier(ast, "value", NodeKind.BindingIdentifier);
  fn.params = ast.new_node(NodeKind.FormalParameters);
  fn.params.items.push(parameter);
  ast.program.body.push(fn);
  assert.equal(fn.params.items.length, 1);
  assert.equal(fn.get("params").get("kind"), "FormalParameters");
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "function created(value) {}");

  const typed = parseSync("function f(value: string) {}", typescript);
  const functionNode = typed.program.body.get(0);
  const value = functionNode.params.items.get(0);
  functionNode.params.items.replace([value]);
  expectCode(generateBuffersSync(typed.buffer, typed.editBuffer ?? new Uint8Array()).code, "function f(value: string) {}", typescript);

  const array = ast.new_node(NodeKind.ArrayExpression);
  const native = array.get("elements");
  native.push(literal(ast, 1));
  native.replace([literal(ast, 2)]);
  assert.equal(array.elements.get(0).value, 2);
  ast.program.body.push(expression(array));
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "function created(value) {} [2];");
});

test("type parameter lists use their native ranges", () => {
  const declaration = parseSync("function f<T>() {}", typescript);
  const typeParameters = declaration.program.body.get(0).typeParameters;
  const parameter = typeParameters.params.get(0);
  typeParameters.params.clear();
  typeParameters.params.push(parameter);
  typeParameters.params.replace([parameter]);
  assert.equal(typeParameters.params.nodeId(0), parameter.nodeId);
  expectCode(generateBuffersSync(...declaration.encode()).code, "function f<T>() {}", typescript);

  const invocation = parseSync("f<string>();", typescript);
  const typeArguments = invocation.program.body.get(0).expression.typeArguments;
  const replacement = invocation.new_node(NodeKind.TsAnyKeyword);
  typeArguments.params.splice(0, 1, replacement);
  typeArguments.params = [replacement];
  assert.equal(typeArguments.params.nodeId(0), replacement.nodeId);
  expectCode(generateBuffersSync(...invocation.encode()).code, "f<any>();", typescript);
});

test("literal changes update raw hints and support Unicode and lone surrogates", () => {
  const string = transform('const value = "old";', (program, ast) => {
    program.body.get(0).declarators.get(0).init.value = "你好\ud800";
  });
  expectCode(string.code, 'const value = "你好\\ud800";');

  const number = transform("const value = 0x1;", (program, ast) => {
    program.body.get(0).declarators.get(0).init.value = 42;
  });
  expectCode(number.code, "const value = 42;");

  const bigint = transform("const value = 1n;", (program, ast) => {
    program.body.get(0).declarators.get(0).init.raw = "7";
  });
  expectCode(bigint.code, "const value = 7n;");

  const different = transform("const value = /a/g;", (program, ast) => {
    program.body.get(0).declarators.get(0).init = literal(ast, "text");
  });
  expectCode(different.code, 'const value = "text";');
});

test("new nodes and growth remain confined to the extension buffer", () => {
  const result = transform("const values = [];", (program, ast) => {
    program.body.get(0).declarators.get(0).init.elements = Array.from(
      { length: 700 },
      (_, i) => literal(ast, `${i}:${"x".repeat(20)}中`),
    );
  });
  const value = parseSync(result.code).program.body.get(0).declarators.get(0).init;
  assert.equal(value.elements.length, 700);
  assert.equal(value.elements.get(699).value, `699:${"x".repeat(20)}中`);
  assert.ok(result.extension.length > result.ast.buffer.length);
});

test("template and RegExp fields and list elisions update storage immediately", () => {
  const template = transform("const value = `old`;", (program, ast) => {
    program.body.get(0).declarators.get(0).init.quasis.get(0).raw = "new";
    program.body.get(0).declarators.get(0).init.quasis.get(0).cooked = "new";
  });
  expectCode(template.code, "const value = `new`;");

  const regex = transform("const value = /old/g;", (program, ast) => {
    program.body.get(0).declarators.get(0).init.pattern = "new";
  });
  expectCode(regex.code, "const value = /new/g;");

  const hole = transform("const value = [1, 2];", (program, ast) => {
    program.body.get(0).declarators.get(0).init.elements.set(0, null);
  });
  expectCode(hole.code, "const value = [, 2];");
});

test("template raw uses native storage in JavaScript and TypeScript", () => {
  const source = "const value = `a\rb\r\nc`;";
  for (const options of [{}, typescript]) {
    const quasi = parseSync(source, options).program.body.get(0).declarators.get(0).init.quasis.get(0);
    assert.equal(quasi.raw, "a\nb\nc");
    quasi.raw = "new";
    assert.equal(quasi.get("raw"), "new");
  }
});

test("TypeScript parameter wrappers and annotations survive incremental edits", () => {
  const annotation = transform(
    'const x: string = "value";',
    (program, ast) => {
      program.body.get(0).declarators.get(0).typeAnnotation.typeAnnotation.type =
        "TsNumberKeyword";
    },
    typescript,
  );
  expectCode(annotation.code, 'const x: number = "value";', typescript);

  const parameter = transform(
    "function f(x: string) {}",
    (program, ast) => {
      const name = identifier(ast, "next", NodeKind.BindingIdentifier);
      const keyword = ast.new_node(NodeKind.TsBooleanKeyword);
      const annotation = ast.new_node(NodeKind.TsTypeAnnotation);
      annotation.set("typeAnnotation", keyword);
      const item = ast.new_node(NodeKind.FormalParameter);
      item.set("pattern", name);
      item.typeAnnotation = annotation;
      program.body.get(0).params.items.push(item);
    },
    typescript,
  );
  expectCode(
    parameter.code,
    "function f(x: string, next: boolean) {}",
    typescript,
  );

  const thisParam = transform(
    "function f(this: C, x: number) {}",
    (program, ast) => {
      program.body.get(0).thisParam.typeAnnotation.typeAnnotation.type = "TsStringKeyword";
    },
    typescript,
  );
  expectCode(
    thisParam.code,
    "function f(this: string, x: number) {}",
    typescript,
  );
});

test("walker replacements and JSX attribute mutations use both buffers", () => {
  const env = transform("import.meta.env.MODE;", (program, ast) => {
    walk(program, {
      MemberExpression(context) {
        const node = context.node;
        if (node.object.type === "MetaProperty")
          { node.object = identifier(ast, "process"); }
      },
    });
  });
  expectCode(env.code, "process.env.MODE;");

  const props = transform(
    '<A data-testid="x" title="before" />;',
    (program, ast) => {
      walk(program, {
        JsxOpeningElement(context) {
          const node = context.node;
          node.attributes = node.attributes.filter(
            (attr) => attr.name.name !== "data-testid",
          );
          const value = node.attributes.get(0).value;
          value.value = "after";
          value.raw = '"after"';
        },
      });
    },
    jsx,
  );
  expectCode(props.code, '<A title="after" />;', jsx);

  const changedType = transform("original;", (program, ast) => {
    const node = program.body.get(0).expression;
    const other = program.body.get(0).expression;
    assert.equal(other.type, "IdentifierReference");
    node.type = "NumericLiteral";
    node.value = 2;
    assert.equal(node.type, "NumericLiteral");
    assert.equal(other.type, "NumericLiteral");
  });
  expectCode(changedType.code, "2;");
});

test("encoding exposes live storage and explicit copies preserve snapshots", () => {
  const ast = parseSync("const x = 1;");
  ast.program.body.get(0).declarators.get(0).id.name = "first";
  const first = ast.encode();
  const saved = first.map(bytes => bytes.slice());
  assert.equal(ast.encode()[1], first[1]);
  ast.program.body.get(0).declarators.get(0).id.name = "second";
  const second = ast.encode();
  expectCode(generateBuffersSync(...saved).code, "const first = 1;");
  expectCode(generateBuffersSync(...second).code, "const second = 1;");
  assert.equal(first[0], second[0]);
});

test("incremental encoding leaves untouched native sibling lists lazy", () => {
  for (const options of [jsx, tsx]) {
    const ast = parseSync("<A><B/><C/><D/><E/></A>;", options);
    walk(ast.program, {
      JsxOpeningElement(context) {
        const node = context.node;
        if (node.name.name === "B") { node.name.name = "Changed"; }
      },
    });
    const created = ast.materializedNodeCount;
    const buffers = ast.encode();
    assert.ok(ast.materializedNodeCount - created < 10);
    expectCode(generateBuffersSync(...buffers).code, "<A><Changed/><C/><D/><E/></A>;", options);

    const rewritten = transform("<A><B title=\"keep\"/></A>;", (program, ast) => {
      walk(program, {
        JsxOpeningElement(context) {
          const node = context.node;
          node.attributes = node.attributes.filter(() => true);
          if (node.name.name === "B") {
            const value = node.attributes.get(0).value;
            value.value = "changed";
            value.raw = '"changed"';
          }
        },
      });
    }, options);
    assert.ok(rewritten.extension.length > 0);
    expectCode(rewritten.code, '<A><B title="changed"/></A>;', options);
  }
});

test("native ancestor references retain holes and typed edits across buffer growth", () => {
  const grown = transform("[0,,call(1),2];", (program, ast) => {
    const array = ast.new_node(NodeKind.ArrayExpression);
    array.elements.replace(Array.from({ length: 700 }, (_, i) => literal(ast, `${i}:中`)));
    program.body.get(0).expression.elements.get(2).arguments.set(0, array);
  });
  const elements = parseSync(grown.code).program.body.get(0).expression.elements;
  assert.equal(elements.get(1), null);
  assert.equal(elements.get(2).arguments.get(0).elements.get(699).value, "699:中");
  assert.equal(elements.get(3).value, 2);

  const changedType = transform("call(original);", (program, ast) => {
    const argument = program.body.get(0).expression.arguments.get(0);
    argument.type = "NumericLiteral";
    argument.value = 7;
  });
  expectCode(changedType.code, "call(7);");

  const replacedCall = parseSync("call(value);");
  const call = replacedCall.program.body.get(0).expression;
  const callee = call.callee;
  const argument = call.arguments.get(0);
  call.type = "NumericLiteral";
  call.value = 9;
  assert.equal(callee.name, "call");
  assert.equal(argument.name, "value");
  expectCode(generateSync(replacedCall.program).code, "9;");

  const changedAnnotation = transform("call<string>(original);", (program, ast) => {
    program.body.get(0).expression.typeArguments.params.get(0).type = "TsNumberKeyword";
  }, typescript);
  expectCode(changedAnnotation.code, "call<number>(original);", typescript);

  const changedLanguage = parseSync("call<string>(original);", typescript);
  changedLanguage.program.body.get(0).expression.callee.name = "renamed";
  assert.throws(() => changedLanguage.encode({ lang: "js" }), TypeError);
  expectCode(generateBuffersSync(...changedLanguage.encode()).code, "renamed<string>(original);", typescript);
});

test("checked buffers reject repeated native child references", () => {
  const buffer = parseBufferSync("[one(), two()];");
  const original = readAst(buffer);
  const array = original.program.body.get(0).expression;
  const [start] = original.range(array.nodeId, "elements");
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const extra = arrayLayout(buffer).extra;
  view.setUint32(extra + (start + 1) * 4, view.getUint32(extra + start * 4, true), true);
  const ast = readAst(buffer);
  assert.throws(() => generateBuffersSync(buffer));
});

test("independent views allocate and edit their own binary nodes", () => {
  const first = parseSync("first;");
  const second = parseSync("second;");
  first.program.body.get(0).expression = identifier(first, "changedFirst");
  second.program.body.get(0).expression = identifier(second, "changedSecond");
  expectCode(generateBuffersSync(first.buffer, first.editBuffer).code, "changedFirst;");
  expectCode(generateBuffersSync(second.buffer, second.editBuffer).code, "changedSecond;");
});

test("Rust checks native field constraints after kind changes", () => {
  const callAst = parseSync('call("source");');
  const argument = callAst.program.body.get(0).expression.arguments.get(0);
  const argumentId = argument.nodeId;
  argument.type = "NumericLiteral";
  argument.value = 1;
  assert.equal(argument.nodeId, argumentId);
  assert.equal(callAst.kind(argumentId), NodeKind.NumericLiteral);
  assert.equal(argument.get("value"), 1);
  argument.set("value", 2);
  assert.equal(argument.value, 2);
  expectCode(generateBuffersSync(...callAst.encode()).code, "call(2);");

  const moduleAst = parseSync('import value from "source";');
  const source = moduleAst.program.body.get(0).source;
  assert.doesNotThrow(() => { source.type = "NumericLiteral"; source.value = 1; });
  assert.doesNotThrow(() => moduleAst.encode());
  assert.throws(() => generateBuffersSync(...moduleAst.encode()));
  source.type = "StringLiteral";
  source.value = "source";
  expectCode(generateSync(moduleAst.program).code, 'import value from "source";');

  const detached = moduleAst.new_node(NodeKind.BindingIdentifier);
  detached.type = "StringLiteral";
  assert.equal(detached.type, "StringLiteral");
  assert.equal(detached.get("name"), undefined);
  assert.throws(() => detached.set("name", "ignored"), TypeError);
  detached.value = "detached";
  assert.equal(moduleAst.kind(detached.nodeId), NodeKind.StringLiteral);
  detached.set("raw", "'detached'");
  assert.equal(detached.get("value"), "detached");
  assert.equal(detached.raw, "'detached'");
});

test("views can be shallow-cloned without materializing descendant fields", () => {
  const ast = parseSync('<A title="value" />;', jsx);
  const before = ast.materializedNodeCount;
  const copy = ast.cloneNode(ast.program);
  assert.ok(copy.body instanceof AstList);
  assert.equal(ast.materializedNodeCount, before + 1);
  assert.equal(copy.body.nodeId(0), ast.program.body.nodeId(0));
  assert.throws(() => generateSync(copy), TypeError);
});

test("Rust rejects cyclic and invalid edits during checked decoding", () => {
  const ast = parseSync("const value = [1];");
  const array = ast.program.body.get(0).declarators.get(0).init;
  const originalElement = array.elements.get(0);
  assert.doesNotThrow(() => array.elements.set(0, array));
  assert.doesNotThrow(() => ast.encode());
  assert.throws(() => generateBuffersSync(...ast.encode()));
  array.elements.set(0, literal(ast, 2));
  assert.equal(originalElement.value, 1);
  expectCode(generateSync(ast.program).code, "const value = [2];");
  const after = ast.buffer.slice();
  assert.throws(() => { array.start = -1; }, RangeError);
  assert.deepEqual(ast.buffer, after);

  const binding = parseSync("const value = 1;");
  assert.doesNotThrow(() => { binding.program.body.get(0).declarators.get(0).id.type = "NumericLiteral"; });
  assert.doesNotThrow(() => binding.encode());
  assert.throws(() => generateSync(binding.program));

  const nested = parseSync("[];");
  const a = nested.program.body.get(0).expression;
  const b = nested.new_node(NodeKind.ArrayExpression), c = nested.new_node(NodeKind.ArrayExpression);
  a.elements = [b];
  b.elements = [c];
  assert.doesNotThrow(() => generateSync(nested.program));
  assert.doesNotThrow(() => { c.elements = [b]; });
  assert.doesNotThrow(() => nested.encode());
  assert.throws(() => generateSync(nested.program));
  c.elements.clear();
  assert.doesNotThrow(() => generateSync(nested.program));

  const reentrant = parseSync("const value = 1;");
  const options = {
    get minify() {
      reentrant.program.body.get(0).declarators.get(0).id.type = "NumericLiteral";
      return true;
    },
  };
  assert.throws(() => generateSync(reentrant.program, options));
});

test("edit inputs are checked before committing binary rows", () => {
  const ast = parseSync("const value = 1;");
  const binding = ast.program.body.get(0).declarators.get(0).id;
  assert.doesNotThrow(() => { binding.name = "\ud800"; });
  assert.equal(binding.name, "\ud800");
  assert.doesNotThrow(() => ast.encode());
  assert.throws(() => generateBuffersSync(...ast.encode()));
  binding.name = "value";

  const body = ast.program.body;
  body.push(expression(literal(ast, 2)));
  assert.throws(() => Object.defineProperty(body, 1, { writable: false }), TypeError);
  assert.doesNotThrow(() => ast.encode());
  expectCode(generateSync(ast.program).code, "const value = 1; 2;");

  const reentrant = parseSync("value;");
  const statement = reentrant.program.body.get(0);
  assert.throws(() => { statement.expression = {
    type: "IdentifierReference",
    get name() { throw new Error("ordinary object must not be read"); },
  }; }, TypeError);
  assert.equal(statement.expression.name, "value");
  expectCode(generateSync(reentrant.program).code, "value;");
});

test("list writes commit before returning and Rust checks the resulting tree", () => {
  const ast = parseSync("`${one}`;");
  const template = ast.program.body.get(0).expression;
  template.expressions.set(0, identifier(ast, "two"));
  const quasi = ast.new_node(NodeKind.TemplateElement);
  quasi.raw = "before";
  quasi.cooked = "before";
  template.quasis.set(0, quasi);
  expectCode(generateSync(ast.program).code, "`before${two}`;");

  const cyclic = parseSync("[];");
  const a = cyclic.program.body.get(0).expression;
  const b = cyclic.new_node(NodeKind.ArrayExpression);
  a.elements.push(b);
  assert.doesNotThrow(() => b.elements.push(a));
  assert.doesNotThrow(() => cyclic.encode());
  assert.throws(() => generateBuffersSync(...cyclic.encode()));
  a.elements.clear();
  b.elements.clear();
  expectCode(generateSync(cyclic.program).code, "[];");

  const pending = parseSync("const a = [], b = [];");
  const [first, second] = pending.program.body.get(0).declarators.map(node => node.init.elements);
  first.push(literal(pending, 1));
  assert.throws(() => second.push({ type: "NumericLiteral", get value() { throw new Error("invalid node"); } }));
  expectCode(generateBuffersSync(pending.buffer, pending.editBuffer).code, "const a = [1], b = [];");
  expectCode(generateSync(pending.program).code, "const a = [1], b = [];");


});

test("checked generation rejects repeated nodes in the current program", () => {
  const ast = parseSync("[value];");
  const elements = ast.program.body.get(0).expression.elements;
  elements.push(elements.get(0));
  assert.doesNotThrow(() => ast.encode());
  assert.throws(() => generateSync(ast.program));
  elements.pop();
  expectCode(generateSync(ast.program).code, "[value];");
});

test("Rust rejects mismatched extensions, invalid references and invalid typed fields", () => {
  const ast = parseSync("const value = 1;");
  ast.program.body.get(0).declarators.get(0).id.name = "newValue";
  const [base, extension] = ast.encode();
  const mismatch = extension.slice();
  new DataView(mismatch.buffer).setUint32(mismatch.byteLength - 16, 0, true);
  assert.throws(() => generateBuffersSync(base, mismatch));
  const badRoot = extension.slice();
  new DataView(badRoot.buffer).setUint32(badRoot.byteLength - 28, 0xfffffffe, true);
  assert.throws(() => generateBuffersSync(base, badRoot));
  const badKind = base.slice();
  const kindOffset = 8 + arrayLayout(base).nodeCount * 8;
  badKind[kindOffset] = 255;
  assert.throws(() => generateBuffersSync(badKind, extension));
  const truncated = parseBufferSync("value;").subarray(0, 4);
  assert.throws(() => generateBuffersSync(truncated));
});

test("detached native subtrees preserve identity when moved", () => {
  const buffer = parseBufferSync("one(); two();");
  const initial = Uint8Array.from(buffer);
  const ast = readAst(buffer);
  const statements = ast.program.body.toArray();
  assert.ok(!statements.at(0).isSame(statements.at(1)));
  statements.at(0).expression.callee.name = "changed";
  assert.equal(statements.at(1).expression.callee.name, "two");
  expectCode(generateSync(ast.program).code, "changed(); two();");
  const expected = viewedNodes(readAst(initial));
  assert.deepEqual(viewedNodes(ast), expected);
  assert.ok(!ast.program.body.get(0).isSame(ast.program.body.get(1)));
  assert.equal(ast.program.body.get(1).expression.callee.name, "two");

  ast.program.body = [];
  assert.deepEqual(viewedNodes(ast), [[ast.rootId, NodeKind.Program]]);

  const block = ast.new_node(NodeKind.BlockStatement);
  ast.program.body = [block];
  block.body = statements;
  assert.deepEqual(viewedNodes(ast), [expected[0], [block.nodeId, NodeKind.BlockStatement], ...expected.slice(1)]);
  expectCode(generateSync(ast.program).code, "{ changed(); two(); }");

  block.body = block.body.filter(node => !node.isSame(statements.at(0)));
  assert.deepEqual(viewedNodes(ast), [expected[0], [block.nodeId, NodeKind.BlockStatement], ...expected.slice(4)]);
  expectCode(generateSync(ast.program).code, "{ two(); }");
});

test("buffer list traversal preserves callbacks and on-demand structural edits", () => {
  const visits = [];
  const edited = transform("first(); second();", (program, ast) => {
    walk(program, {
      ExpressionStatement(context) {
        const node = context.node;
        const name = node.expression.type === "CallExpression" ? node.expression.callee.name : node.expression.name;
        visits.push(name);
        if (name === "first") {
          context.insertBefore(expression(identifier(ast, "before")));
          context.insertAfter(expression(identifier(ast, "after")));
        } else if (name === "second") { context.remove(); }
      },
    });
  });
  assert.deepEqual(visits, ["first", "after", "second"]);
  expectCode(edited.code, "before; first(); after;");

  const appendedVisits = [];
  const appended = transform("first();", (program, ast) => {
    walk(program, {
      ExpressionStatement(context) {
        const node = context.node;
        const name = node.expression.type === "CallExpression" ? node.expression.callee.name : node.expression.name;
        appendedVisits.push(name);
        if (name === "first") { context.parent.body.push(expression(identifier(ast, "last"))); }
      },
    });
  });
  assert.deepEqual(appendedVisits, ["first", "last"]);
  expectCode(appended.code, "first(); last;");

  const replaced = transform("first();", (program, ast) => {
    walk(program, {
      ExpressionStatement(context) {
        context.parent.body = [expression(identifier(ast, "replacement"))];
        assert.throws(() => context.remove());
        context.stop();
      },
    });
  });
  expectCode(replaced.code, "replacement;");
});
