import type {
  Node,
  Program,
  Expr,
  FormalParameterItem,
  IdentifierReference,
  ExpressionStatement,
} from "@swc-next/ast2";
import {
  parseSync,
  parseBufferSync,
  generateSync,
  generateBuffersSync,
  walk,
} from "../dist/ffi2.js";
import { readAst, NodeKind, type NodeView, type AstNode, type AstNodeInput, type AstList } from "@swc-next/ast2";

const ast = parseSync("function f(value: string) { return value; }", {
  lang: "ts",
});
const program: AstNode<Program> = ast.program;
const body: AstList<AstNode<Program["body"][number]>> = program.body;
const node: AstNode = body.get(0)!;
if (node.type === "Function") {
  const params: AstList<AstNode<FormalParameterItem>> = node.params.items;
  params.map((parameter) => parameter.type);
  if (node.id) { node.id.name = "renamed"; }
}
walk(program, {
  IdentifierReference(context) {
    const name: string = context.node.name;
    void name;
  },
});
walk(program, {
  IdentifierReference(context) {
    context.replace({ kind: NodeKind.IdentifierReference, name: "renamed" });
    context.insertBefore({ kind: NodeKind.EmptyStatement });
    context.insertAfter([{ kind: NodeKind.EmptyStatement }]);
  },
});
generateSync(program);
const [original, extension]: [Uint8Array, Uint8Array] = ast.encode();
generateBuffersSync(original, extension, { minify: true });
const shallow = ast.cloneNode(program);
shallow.body = [];
const editable: Program = ast.materialize(program);
editable.body.push(ast.materialize(program.body.get(0)!));
const buffer: Uint8Array = parseBufferSync("value;");
const opened: AstNode<Program> = readAst(buffer).program;
const identifier = ast.new_node<IdentifierReference>(NodeKind.IdentifierReference);
identifier.name = "created";
const created: AstNode<IdentifierReference> = identifier;
const statement = ast.new_node<ExpressionStatement>(NodeKind.ExpressionStatement);
statement.expression = identifier;
// @ts-expect-error List mutations only accept buffer nodes.
statement.ast.program.body.push({ type: "EmptyStatement", start: 0, end: 0 });
const removed: (AstNode | null)[] = ast.spliceList(ast.program, "body", 0, 0, statement);
const editBuffer: Uint8Array | null = ast.editBuffer;
const count: number = ast.totalNodeCount;
void [created, removed, editBuffer, count];
const firstStatement = opened.body.get(0)!;
if (firstStatement.type === "ExpressionStatement") {
  const expression: AstNode<Expr> = firstStatement.expression;
  void expression;
}

const handle: NodeView<Program> = program;
const fromHandle: AstNodeInput<Program> = ast.cloneNode(handle);
const fromView: AstNodeInput<Program> = ast.cloneNode(program);
const serializedList: AstNode[] = ast.program.body.toJSON();
const json: unknown = handle.toJSON();
void [fromHandle, fromView, serializedList, json];
// @ts-expect-error Generation requires a root view, not a shallow copy.
generateSync(fromHandle);
