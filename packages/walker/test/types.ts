import { walk, type AstNode, type WalkHooks } from "@swc-next/walker";

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
walk(root, {
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
// @ts-expect-error Unknown node type
walk(root, { Typo() {} });
const invalid: WalkHooks<{ count: number }> = {
  Identifier(node, ctx) {
    // @ts-expect-error Wrong state type
    ctx.state.count = "wrong";
  },
};
void invalid;
