// Task-specific baseline for walk.bench.mjs's generated JS functions.
// No walker, visitor keys, hooks or context from any package are used.
// Unsupported node types throw: this is not a general JS/TS transformer.
export function editByHand(root) {
  let edits = 0;

  function editList(nodes) {
    for (let i = 0; i < nodes.length; i++) {
      if (nodes[i] != null) i = editNode(nodes[i], null, null, nodes, i);
    }
  }

  function editNode(node, parent = null, key = null, list = null, index = 0) {
    switch (node.type) {
      case "Program":
      case "BlockStatement":
        editList(node.body);
        break;
      case "FunctionDeclaration":
        if (node.id) editNode(node.id, node, "id");
        editList(node.params);
        editNode(node.body, node, "body");
        break;
      case "DebuggerStatement":
        edits++;
        if (!list) throw new Error("Expected debugger in a statement list");
        list.splice(index, 1);
        return index - 1;
      case "ExpressionStatement":
        edits++;
        if (
          node.expression.type === "CallExpression" &&
          node.expression.callee.name === "keep"
        ) {
          if (!list) throw new Error("Expected keep() in a statement list");
          list.splice(index, 0, { type: "EmptyStatement" });
          index++; // The inserted-before node is not visited.
          list.splice(index + 1, 0, { type: "EmptyStatement" });
        }
        editNode(node.expression, node, "expression");
        break;
      case "CallExpression":
        editNode(node.callee, node, "callee");
        editList(node.arguments);
        break;
      case "ReturnStatement":
        if (node.argument) editNode(node.argument, node, "argument");
        break;
      case "BinaryExpression":
        editNode(node.left, node, "left");
        editNode(node.right, node, "right");
        break;
      case "Literal":
        edits++;
        if (node.value === 1) {
          const replacement = { ...node, value: 2 };
          if (list) list[index] = replacement;
          else parent[key] = replacement;
        }
        break;
      case "Identifier":
      case "EmptyStatement":
        break;
      default:
        throw new Error(`Unsupported handwritten fixture node: ${node.type}`);
    }
    return index;
  }

  editNode(root);
  return edits;
}

// Independent, untimed verification for this fixture's AST subset. Returns the
// same position trace as the library cases without calling their visitor APIs.
export function traceByHand(root) {
  const rows = [];
  function record(node, key = null, index = null) {
    rows.push([node.type, node.start, node.end, key, index]);
    switch (node.type) {
      case "Program":
      case "BlockStatement":
        recordList(node.body, "body");
        break;
      case "FunctionDeclaration":
        if (node.id) record(node.id, "id");
        recordList(node.params, "params");
        record(node.body, "body");
        break;
      case "ExpressionStatement":
        record(node.expression, "expression");
        break;
      case "CallExpression":
        record(node.callee, "callee");
        recordList(node.arguments, "arguments");
        break;
      case "ReturnStatement":
        if (node.argument) record(node.argument, "argument");
        break;
      case "BinaryExpression":
        record(node.left, "left");
        record(node.right, "right");
        break;
      case "DebuggerStatement":
      case "Literal":
      case "Identifier":
      case "EmptyStatement":
        break;
      default:
        throw new Error(`Unsupported handwritten fixture node: ${node.type}`);
    }
  }
  function recordList(nodes, key) {
    for (let i = 0; i < nodes.length; i++) {
      if (nodes[i] != null) record(nodes[i], key, i);
    }
  }
  record(root);
  return rows;
}
