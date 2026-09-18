import { parse } from "acorn";
import type * as ESTree from "estree";
import { parseSync, printSync, type Program, type AST } from "../../src-js/index.js";
import { printSync as printCodegen } from "@swc-next/codegen";
import { parseSync as parseParser } from "@swc-next/parser";
import { printSync as printBrowser, parseSync as parseBrowser } from "@swc-next/core-browser";
import { encode } from "@swc-next/encoder";
import { decode } from "@swc-next/decoder";

const acornProgram: Program = parse("function f(x) { return x + 1; }", { ecmaVersion: "latest" });
printSync(acornProgram);
printCodegen(parse("export const x = 1", { ecmaVersion: "latest", sourceType: "module" }));
printBrowser(acornProgram);
encode(acornProgram);

// The ESTree declarations also permit synthetic nodes with no positions.
declare const estreeProgram: ESTree.Program;
const compatible: Program = estreeProgram;
printCodegen(compatible);

const parsed: Program = parseSync("const x = 1").program;
const focused: Program = parseParser("const x = 1").program;
const browser: Program = parseBrowser("const x = 1").program;
const decoded: Program = decode(encode(parsed), "const x = 1").program;
printSync(focused);
printSync(browser);
printSync(decoded);

const declaration: AST.TSDeclareFunction = {
  type: "TSDeclareFunction",
  id: { type: "Identifier", name: "f" },
  generator: false,
  async: false,
  params: [],
  typeParameters: null,
  returnType: null,
  declare: true,
  body: null,
  expression: false,
};
printSync({ type: "Program", sourceType: "module", body: [declaration] });

// Keep useful structure checks despite accepting multiple ESTree producers.
// @ts-expect-error The discriminant is required.
const missingType: Program = { body: [], sourceType: "module" };
// @ts-expect-error Statement lists cannot contain primitive values.
printSync({ type: "Program", sourceType: "module", body: [42] });
// @ts-expect-error A TS declaration cannot acquire a function body.
declaration.body = { type: "BlockStatement", body: [] };
for (const statement of parsed.body) {
  if (statement.type === "VariableDeclaration") {
    statement.declarations[0].id;
    // @ts-expect-error Narrowing retains node-specific fields.
    statement.arguments;
  }
}
