import assert from "node:assert/strict";
import test from "node:test";
import {
  parseAstSync,
  parseSync,
  parseBufferSync,
  readAst,
  NodeView,
  generateSync,
  generateBuffersSync,
  walk,
} from "../src-js/index.js";

const typescript = { lang: "ts" };
const jsx = { lang: "jsx" };
const tsx = { lang: "tsx" };
const identifier = (name) => ({ type: "Identifier", name, start: 0, end: 0 });
const literal = (value) => ({
  type: "Literal",
  value,
  raw: typeof value === "bigint" ? `${value}n` : JSON.stringify(value),
  start: 0,
  end: 0,
});
const expression = (value) => ({
  type: "ExpressionStatement",
  expression: value,
  start: 0,
  end: 0,
});

function transform(source, change, options = {}) {
  const ast = parseAstSync(source, options);
  const snapshot = ast.buffer.slice();
  change(ast.program, ast);
  const [buffer, extension] = ast.encode();
  assert.equal(buffer, ast.buffer);
  assert.deepEqual(buffer, snapshot);
  const result = generateBuffersSync(buffer, extension);
  assert.equal(generateSync(ast.program).code, result.code);
  return { ast, extension, code: result.code };
}

function expectCode(actual, source, options) {
  const { program } = parseSync(source, options);
  assert.equal(actual, generateSync(program).code);
}

function nativeTraversal(ast) {
  const nodes = [];
  ast.traverse((id, kind) => nodes.push([id, kind]));
  return nodes;
}

test("unchanged views use the original buffer with an empty extension", () => {
  const ast = parseAstSync('const 文 = "你好";');
  const [buffer, extension] = ast.encode();
  assert.equal(buffer, ast.buffer);
  assert.equal(extension.length, 0);
  assert.equal(ast.materializedNodeCount, 0);
  expectCode(generateBuffersSync(buffer, extension).code, 'const 文 = "你好";');
  ast.materialize();
  assert.equal(ast.encode()[1].length, 0);
});

test("setters write stable-ID override rows without rebuilding ancestors", () => {
  const result = transform(
    "let x = 1; function keep() { return longCall(a, b, c); }",
    (program) => {
      program.body[0].declarations[0].id.name = "renamed";
    },
  );
  expectCode(
    result.code,
    "let renamed = 1; function keep() { return longCall(a, b, c); }",
  );
  const view = new DataView(result.extension.buffer);
  assert.equal(view.getUint32(0, true), 0x33455753);
  assert.equal(view.getUint32(32, true), 0);
  assert.equal(view.getUint32(36, true), 1);
});

test("node creation and setters publish bytes before encode and preserve old snapshots", () => {
  const ast = parseAstSync("original;");
  const base = ast.buffer.slice();
  const value = ast.createNode(literal(42));
  assert.equal(ast._edits, null);
  assert.equal(value.value, 42);
  assert.equal(new DataView(ast.editBuffer.buffer).getUint32(32, true), 1);
  const statement = ast.program.body[0];
  const statementId = statement.nodeId;
  const previous = statement.expression;
  statement.expression = value;
  assert.equal(ast.program.body[0].nodeId, statementId);
  assert.equal(previous.parent, null);
  assert.ok(NodeView.isSame(value.parent, statement));
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "42;");
  assert.deepEqual(ast.buffer, base);

  const rows = ast._arena.rowCount;
  const first = ast.encode();
  assert.equal(ast._arena.rowCount, rows);
  const frozen = first[1].slice();
  value.value = 100.5;
  assert.equal(ast._edits, null);
  assert.equal(ast._arena.rowCount, rows);
  assert.deepEqual(first[1], frozen);
  expectCode(generateBuffersSync(...first).code, "42;");
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "100.5;");
  assert.deepEqual(ast.buffer, base);
});

test("list editing and foreign node import write independent binary data", () => {
  const ast = parseAstSync("original;");
  const foreign = parseAstSync("aMuchLongerIdentifier;");
  const imported = ast.createNode(foreign.program.body[0].expression);
  assert.equal(imported.start, 0);
  assert.equal(imported.end, 0);
  ast.spliceList(ast.program, "body", 0, 1, expression(imported));
  expectCode(generateBuffersSync(ast.buffer, ast.editBuffer).code, "aMuchLongerIdentifier;");
  foreign.program.body[0].expression.name = "changedElsewhere";
  assert.equal(imported.name, "aMuchLongerIdentifier");
});

test("ordinary arrays support insertion, removal, replacement and ordering", () => {
  const inserted = transform("a(); b();", (program) => {
    program.body.splice(1, 0, expression(literal(3)));
  });
  expectCode(inserted.code, "a(); 3; b();");

  const removed = transform("a(); b();", (program) => {
    program.body.splice(0, 1);
  });
  expectCode(removed.code, "b();");

  const replaced = transform("a();", (program) => {
    program.body[0] = expression(identifier("replacement"));
  });
  expectCode(replaced.code, "replacement;");

  const reordered = transform("a(); b();", (program) => {
    program.body.reverse();
  });
  expectCode(reordered.code, "b(); a();");
});

test("literal changes update raw hints and support Unicode and lone surrogates", () => {
  const string = transform('const value = "old";', (program) => {
    program.body[0].declarations[0].init.value = "你好\ud800";
  });
  expectCode(string.code, 'const value = "你好\\ud800";');

  const number = transform("const value = 0x1;", (program) => {
    program.body[0].declarations[0].init.value = 42;
  });
  expectCode(number.code, "const value = 42;");

  const bigint = transform("const value = 1n;", (program) => {
    program.body[0].declarations[0].init.value = 7n;
  });
  expectCode(bigint.code, "const value = 7n;");

  const different = transform("const value = /a/g;", (program) => {
    program.body[0].declarations[0].init.value = "text";
  });
  expectCode(different.code, 'const value = "text";');
});

test("new nodes and growth remain confined to the extension buffer", () => {
  const result = transform("const values = [];", (program) => {
    program.body[0].declarations[0].init.elements = Array.from(
      { length: 700 },
      (_, i) => literal(`${i}:${"x".repeat(20)}中`),
    );
  });
  const value = parseSync(result.code).program.body[0].declarations[0].init;
  assert.equal(value.elements.length, 700);
  assert.equal(value.elements[699].value, `699:${"x".repeat(20)}中`);
  assert.ok(result.extension.length > result.ast.buffer.length);
});

test("nested object edits and sparse array deletions are detected at encoding", () => {
  const template = transform("const value = `old`;", (program) => {
    program.body[0].declarations[0].init.quasis[0].value.raw = "new";
    program.body[0].declarations[0].init.quasis[0].value.cooked = "new";
  });
  expectCode(template.code, "const value = `new`;");

  const regex = transform("const value = /old/g;", (program) => {
    program.body[0].declarations[0].init.regex.pattern = "new";
  });
  expectCode(regex.code, "const value = /new/g;");

  const hole = transform("const value = [1, 2];", (program) => {
    delete program.body[0].declarations[0].init.elements[0];
  });
  expectCode(hole.code, "const value = [, 2];");
});

test("TypeScript parameter wrappers and annotations survive incremental edits", () => {
  const annotation = transform(
    'const x: string = "value";',
    (program) => {
      program.body[0].declarations[0].id.typeAnnotation.typeAnnotation.type =
        "TSNumberKeyword";
    },
    typescript,
  );
  expectCode(annotation.code, 'const x: number = "value";', typescript);

  const parameter = transform(
    "function f(x: string) {}",
    (program) => {
      program.body[0].params.push({
        ...identifier("next"),
        optional: false,
        decorators: [],
        typeAnnotation: {
          type: "TSTypeAnnotation",
          start: 0,
          end: 0,
          typeAnnotation: { type: "TSBooleanKeyword", start: 0, end: 0 },
        },
      });
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
    (program) => {
      program.body[0].params[0].name = "context";
    },
    typescript,
  );
  expectCode(
    thisParam.code,
    "function f(context: C, x: number) {}",
    typescript,
  );
});

test("walker replacements and JSX attribute mutations use both buffers", () => {
  const env = transform("import.meta.env.MODE;", (program) => {
    walk(program, {
      MemberExpression(node) {
        if (node.object.type === "MetaProperty")
          node.object = identifier("process");
      },
    });
  });
  expectCode(env.code, "process.env.MODE;");

  const props = transform(
    '<A data-testid="x" title="before" />;',
    (program) => {
      walk(program, {
        JSXOpeningElement(node) {
          node.attributes = node.attributes.filter(
            (attr) => attr.name.name !== "data-testid",
          );
          node.attributes[0].value.value = "after";
        },
      });
    },
    jsx,
  );
  expectCode(props.code, '<A title="after" />;', jsx);

  const changedType = transform("original;", (program) => {
    const node = program.body[0].expression;
    const other = program.body[0].expression;
    assert.equal(other.type, "Identifier");
    node.type = "Literal";
    node.value = 2;
    assert.equal(node.type, "Literal");
    assert.equal(other.type, "Literal");
  });
  expectCode(changedType.code, "2;");
});

test("encoding is repeatable and earlier buffer pairs remain valid", () => {
  const ast = parseAstSync("const x = 1;");
  ast.program.body[0].declarations[0].id.name = "first";
  const first = ast.encode();
  assert.deepEqual(ast.encode()[1], first[1]);
  ast.program.body[0].declarations[0].id.name = "second";
  const second = ast.encode();
  expectCode(generateBuffersSync(...first).code, "const first = 1;");
  expectCode(generateBuffersSync(...second).code, "const second = 1;");
  assert.equal(first[0], second[0]);
});

test("incremental encoding leaves untouched native sibling lists lazy", () => {
  for (const options of [jsx, tsx]) {
    const ast = parseAstSync("<A><B/><C/><D/><E/></A>;", options);
    walk(ast.program, {
      JSXOpeningElement(node) {
        if (node.name.name === "B") node.name.name = "Changed";
      },
    });
    const created = ast.materializedNodeCount;
    const buffers = ast.encode();
    assert.ok(ast.materializedNodeCount - created < 10);
    expectCode(generateBuffersSync(...buffers).code, "<A><Changed/><C/><D/><E/></A>;", options);

    const rewritten = transform("<A><B title=\"keep\"/></A>;", (program) => {
      walk(program, {
        JSXOpeningElement(node) {
          node.attributes = node.attributes.filter(() => true);
          if (node.name.name === "B") node.attributes[0].value.value = "changed";
        },
      });
    }, options);
    assert.ok(rewritten.extension.length > 0);
    expectCode(rewritten.code, '<A><B title="changed"/></A>;', options);
  }
});

test("native ancestor references retain holes and typed edits across buffer growth", () => {
  const grown = transform("[0,,call(1),2];", (program) => {
    program.body[0].expression.elements[2].arguments[0] = {
      type: "ArrayExpression", start: 0, end: 0,
      elements: Array.from({ length: 700 }, (_, i) => literal(`${i}:中`)),
    };
  });
  const elements = parseSync(grown.code).program.body[0].expression.elements;
  assert.equal(elements[1], null);
  assert.equal(elements[2].arguments[0].elements[699].value, "699:中");
  assert.equal(elements[3].value, 2);

  const changedType = transform("call(original);", (program) => {
    const argument = program.body[0].expression.arguments[0];
    argument.type = "Literal";
    argument.value = 7;
  });
  expectCode(changedType.code, "call(7);");

  const replacedCall = parseAstSync("call(value);");
  const call = replacedCall.program.body[0].expression;
  const callee = call.callee;
  const argument = call.arguments[0];
  call.type = "Literal";
  call.value = 9;
  assert.equal(callee.parent, null);
  assert.equal(argument.parent, null);
  expectCode(generateSync(replacedCall.program).code, "9;");

  const changedAnnotation = transform("call<string>(original);", (program) => {
    program.body[0].expression.typeArguments.params[0].type = "TSNumberKeyword";
  }, typescript);
  expectCode(changedAnnotation.code, "call<number>(original);", typescript);

  const changedLanguage = parseAstSync("call<string>(original);", typescript);
  changedLanguage.program.body[0].expression.callee.name = "renamed";
  assert.throws(() => changedLanguage.encode({ lang: "js" }), TypeError);
  expectCode(generateBuffersSync(...changedLanguage.encode()).code, "renamed<string>(original);", typescript);
});

test("native list copies remap shared child occurrences independently", () => {
  const buffer = parseBufferSync("[one(), two()];");
  const original = readAst(buffer);
  const array = original.program.body[0].expression;
  const [start] = original.range(array.nodeId, "elements");
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const extra = view.getUint32(36, true);
  view.setUint32(extra + (start + 1) * 8, view.getUint32(extra + start * 8, true), true);
  const ast = readAst(buffer);
  const elements = ast.program.body[0].expression.elements;
  assert.ok(!NodeView.isSame(elements[0], elements[1]));
  elements[1].callee.name = "changed";
  expectCode(generateBuffersSync(...ast.encode()).code, "[one(), changed()];");
  assert.equal(elements[0].callee.name, "one");
});

test("nested encoding and failed edits keep transaction state independent", () => {
  for (const options of [{}, typescript]) {
    const inner = parseAstSync("let inner = 1;", options);
    inner.program.body[0].declarations[0].id.name = "renamed";
    const original = inner.encode();
    const saved = original[1].slice();
    const outer = parseAstSync("let outer = 0;", options);
    let nested;
    outer.program.body[0].declarations[0].init = {
      ...literal(2),
      get value() {
        nested = inner.encode();
        return 2;
      },
    };
    const pair = outer.encode();
    assert.equal(pair[0], outer.buffer);
    assert.equal(nested[0], inner.buffer);
    assert.deepEqual(original[1], saved);
    expectCode(generateBuffersSync(...pair).code, "let outer = 2;", options);
    expectCode(generateBuffersSync(...nested).code, "let renamed = 1;", options);
    expectCode(generateBuffersSync(...original).code, "let renamed = 1;", options);

    const declaration = outer.program.body[0].declarations[0];
    const retained = declaration.init;
    assert.throws(() => {
      outer.program.body[0].declarations[0].init = {
        ...literal(3),
        get value() { throw new Error("edit getter failed"); },
      };
    });
    assert.ok(NodeView.isSame(declaration.init, retained));
    assert.ok(NodeView.isSame(retained.parent, declaration));
    expectCode(generateBuffersSync(...outer.encode()).code, "let outer = 2;", options);
    outer.program.body[0].declarations[0].init = literal(4);
    expectCode(generateBuffersSync(...outer.encode()).code, "let outer = 4;", options);
  }
});

test("views can be shallow-cloned without materializing descendant fields", () => {
  const result = transform("const x = 1;", (program, ast) => {
    const original = program.body[0].declarations[0].init;
    program.body[0].declarations[0].init = {
      ...ast.cloneNode(original),
      value: 3,
      raw: "3",
    };
  });
  expectCode(result.code, "const x = 3;");
  const jsxAst = parseAstSync('<A title="value" />;', jsx);
  expectCode(generateSync(jsxAst.cloneNode(jsxAst.program)).code, '<A title="value" />;', jsx);
});

test("cyclic or invalid edits fail without modifying the base buffer", () => {
  const ast = parseAstSync("const value = [1];");
  const before = ast.buffer.slice();
  const array = ast.program.body[0].declarations[0].init;
  const originalElement = array.elements[0];
  array.elements[0] = array;
  assert.throws(() => ast.encode(), TypeError);
  assert.ok(NodeView.isSame(originalElement.parent, array));
  assert.deepEqual(ast.buffer, before);
  array.elements[0] = literal(2);
  expectCode(generateSync(ast.program).code, "const value = [2];");
  assert.equal(originalElement.parent, null);
  assert.throws(() => { array.start = -1; }, RangeError);
  assert.deepEqual(ast.buffer, before);

  const binding = parseAstSync("const value = 1;");
  binding.program.body[0].declarations[0].id.type = "Literal";
  assert.throws(() => generateSync(binding.program));

  const shared = parseAstSync("const a = [], b = [], c = [], d = [];");
  const [a, b, c, d] = shared.program.body[0].declarations.map(node => node.init);
  a.elements = [b];
  c.elements = [b];
  assert.doesNotThrow(() => generateSync(shared.program));
  c.elements = [];
  assert.notEqual(shared.parentId(b.nodeId), c.nodeId);
  b.elements = [c];
  assert.ok(NodeView.isSame(c.parent, b));
  d.elements = [c];
  assert.doesNotThrow(() => generateSync(shared.program));
  c.elements = [b];
  assert.throws(() => generateSync(shared.program));

  const reentrant = parseAstSync("const value = 1;");
  const options = {
    get minify() {
      reentrant.program.body[0].declarations[0].id.type = "Literal";
      return true;
    },
  };
  assert.throws(() => generateSync(reentrant.program, options));
});

test("Rust rejects mismatched extensions, invalid references and invalid typed fields", () => {
  const ast = parseAstSync("const value = 1;");
  ast.program.body[0].declarations[0].id.name = "newValue";
  const [base, extension] = ast.encode();
  const mismatch = extension.slice();
  new DataView(mismatch.buffer).setUint32(16, 0, true);
  assert.throws(() => generateBuffersSync(base, mismatch));
  const badRoot = extension.slice();
  new DataView(badRoot.buffer).setUint32(48, 0xfffffffe, true);
  assert.throws(() => generateBuffersSync(base, badRoot));
  const badKind = extension.slice();
  const invalid = new DataView(badKind.buffer);
  invalid.setUint8(invalid.getUint32(56, true) + 8, 255);
  assert.throws(() => generateBuffersSync(base, badKind));
  const truncated = parseBufferSync("value;").subarray(0, 72);
  assert.throws(() => generateBuffersSync(truncated));
});

test("shared native nodes keep independent editable occurrences", () => {
  const buffer = parseBufferSync("one(); two();");
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const count = view.getUint32(20, true);
  const root = view.getUint32(24, true);
  const payloads = Math.ceil((view.getUint32(28, true) + count * 9) / 8) * 8;
  const extra = view.getUint32(36, true);
  const blockOffset = view.getUint32(payloads + root * 16 + 12, true);
  const start = view.getUint32(extra + blockOffset * 8, true);
  const first = view.getUint32(extra + start * 8, true);
  view.setUint32(extra + (start + 1) * 8, first, true);
  const ast = readAst(buffer);
  const statements = ast.program.body;
  assert.ok(!NodeView.isSame(statements[0], statements[1]));
  statements[0].expression.callee.name = "changed";
  assert.equal(statements[1].expression.callee.name, "one");
  expectCode(generateSync(ast.program).code, "changed(); one();");
  const expected = nativeTraversal(readAst(buffer));
  assert.deepEqual(nativeTraversal(ast), expected);
  assert.ok(!NodeView.isSame(ast.program.body[0], ast.program.body[1]));
  assert.equal(ast.program.body[1].expression.callee.name, "one");

  ast.program.body = [];
  assert.equal(statements[0].parent, null);
  assert.equal(statements[1].parent, null);
  assert.deepEqual(nativeTraversal(ast), expected);

  const block = ast.createNode({ type: "BlockStatement", body: [] });
  ast.program.body = [block];
  block.body = statements;
  assert.ok(NodeView.isSame(statements[0].parent, block));
  assert.ok(NodeView.isSame(statements[1].parent, block));
  assert.deepEqual(nativeTraversal(ast), expected);
  expectCode(generateSync(ast.program).code, "{ changed(); one(); }");

  block.body = block.body.filter(node => !NodeView.isSame(node, statements[0]));
  assert.equal(statements[0].parent, null);
  assert.ok(NodeView.isSame(statements[1].parent, block));
  assert.deepEqual(nativeTraversal(ast), expected);
  expectCode(generateSync(ast.program).code, "{ one(); }");
});

test("buffer list traversal preserves callbacks and on-demand structural edits", () => {
  const visits = [];
  const edited = transform("first(); second();", (program) => {
    walk(program, {
      ExpressionStatement(node, context) {
        const name = node.expression.type === "CallExpression" ? node.expression.callee.name : node.expression.name;
        visits.push(name);
        if (name === "first") {
          context.insertBefore(expression(identifier("before")));
          context.insertAfter(expression(identifier("after")));
        } else if (name === "second") context.remove();
      },
    });
  });
  assert.deepEqual(visits, ["first", "after", "second"]);
  expectCode(edited.code, "before; first(); after;");

  const appendedVisits = [];
  const appended = transform("first();", (program) => {
    walk(program, {
      ExpressionStatement(node, context) {
        const name = node.expression.type === "CallExpression" ? node.expression.callee.name : node.expression.name;
        appendedVisits.push(name);
        if (name === "first") context.parent.body.push(expression(identifier("last")));
      },
    });
  });
  assert.deepEqual(appendedVisits, ["first", "last"]);
  expectCode(appended.code, "first(); last;");

  const replaced = transform("first();", (program) => {
    walk(program, {
      ExpressionStatement(_node, context) {
        context.parent.body = [expression(identifier("replacement"))];
        assert.throws(() => context.remove());
        context.stop();
      },
    });
  });
  expectCode(replaced.code, "replacement;");
});
