import { transformSync, type JsPlugin } from "../src-js/index.js";

function removeDebugger(): JsPlugin {
  return {
    name: "remove-debugger",
    visitors: {
      Program(program) {
        const imports = program.body.filter(
          (node) => node.type === "ImportDeclaration",
        );
        console.log(imports.map((node) => node.source.value));
      },
      DebuggerStatement(_, ctx) {
        if (ctx.index !== null) ctx.remove();
        else ctx.replace({ type: "EmptyStatement" });
      },
    },
  };
}

const { code } = transformSync(
  'import value from "dependency"; const answer: number = value; debugger;',
  {
    filename: "input.ts",
    isModule: true,
    jsc: {
      parser: { syntax: "typescript" },
      target: "esnext",
      transform: { verbatimModuleSyntax: true },
    },
    plugins: [removeDebugger()],
  },
);
console.log(code);
