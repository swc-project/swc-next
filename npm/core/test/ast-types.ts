import {
  walk,
  transformSync,
  type AstNode,
  type WalkHooks,
  type TransformPlugin,
} from "../src-js/index.js";
import { walk as standaloneWalk } from "@swc-next/core/ast";

const root: AstNode = { type: "Program", body: [] };
walk(
  root,
  {
    Identifier(node, ctx) {
      const type: "Identifier" = node.type;
      ctx.state.count++;
      ctx.replace({ type: "Literal", value: true });
      void type;
    },
  },
  { count: 0 },
);
standaloneWalk(root, {
  Function(node) {
    const type:
      | "FunctionDeclaration"
      | "FunctionExpression"
      | "ArrowFunctionExpression"
      | "TSDeclareFunction"
      | "TSEmptyBodyFunctionExpression" = node.type;
    void type;
  },
});

// Consumers may provide a richer discriminated union without coupling the
// runtime to another parser's AST package.
type Nodes =
  | { type: "Identifier"; name: string }
  | { type: "Literal"; value: number };
const hooks: WalkHooks<{ count: number }, Nodes> = {
  Identifier(node, ctx) {
    node.name.toUpperCase();
    ctx.state.count++;
  },
  Literal(node) {
    node.value.toFixed();
  },
};
walk<AstNode, { count: number }, Nodes>(root, hooks, { count: 0 });
const plugin: TransformPlugin<{ count: number }> = {
  state: { count: 0 },
  visitor: {
    Identifier(node, ctx) {
      ctx.state.count++;
    },
  },
};
const result = transformSync("run();", { plugins: [plugin] });
result.code.toUpperCase();
// @ts-expect-error Unknown node type
standaloneWalk(root, { Typo() {} });
const invalid: WalkHooks<{ count: number }> = {
  Identifier(node, ctx) {
    // @ts-expect-error Wrong state type
    ctx.state.count = "wrong";
  },
};
void invalid;
