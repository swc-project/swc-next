import type {
  Node,
  Program,
  Expression,
  FormalParameters,
} from "@swc-next/types";
import {
  parseAstSync,
  parseBufferSync,
  generateSync,
  generateBuffersSync,
  walk,
} from "../src-js/index.js";
import { readAst, NodeKind, type NodeView } from "@swc-next/ast";

const ast = parseAstSync("function f(value: string) { return value; }", {
  lang: "ts",
});
const program: Program = ast.program;
const body: Program["body"] = program.body;
const node: Node = body[0];
if (node.type === "FunctionDeclaration") {
  const params: FormalParameters = node.params;
  params.map((parameter) => parameter.type);
  if (node.id) node.id.name = "renamed";
}
walk(program, {
  Identifier(node) {
    const name: string = node.name;
    void name;
  },
});
generateSync(program);
const [original, extension]: [Uint8Array, Uint8Array] = ast.encode();
generateBuffersSync(original, extension, { minify: true });
const shallow: Program = ast.cloneNode(program);
shallow.body = [];
const editable: Program = ast.materialize(program);
editable.body.push(program.body[0]);
const buffer: Uint8Array = parseBufferSync("value;");
const opened: Program = readAst(buffer).program;
const identifier = ast.createNode({ type: "Identifier", name: "created", start: 0, end: 0 }, NodeKind.IdentifierReference);
const created: Node & NodeView = identifier;
const statement = ast.createNode({ type: "ExpressionStatement", expression: identifier, start: 0, end: 0 });
const removed: Node[] = ast.spliceList(ast.program, "body", 0, 0, statement);
const editBuffer: Uint8Array | null = ast.editBuffer;
const count: number = ast.totalNodeCount;
void [created, removed, editBuffer, count];
if (opened.body[0].type === "ExpressionStatement") {
  const expression: Expression = opened.body[0].expression;
  void expression;
}
