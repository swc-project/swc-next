import { parseSync, printSync, type Program, type AST } from "../../src-js/index.js";
import { printSync as printCodegen } from "@swc-next/codegen";
import { parseSync as parseParser } from "@swc-next/parser";
import { printSync as printBrowser, parseSync as parseBrowser } from "@swc-next/core-browser";
import { encode } from "@swc-next/encoder";
import { decode } from "@swc-next/decoder";

const parsed: Program = parseSync("const x = 1").program;
const focused: Program = parseParser("const x = 1").program;
const browser: Program = parseBrowser("const x = 1").program;
const decoded: Program = decode(encode(parsed), "const x = 1", new TextEncoder().encode("const x = 1")).program;
printSync(focused);
printSync(browser);
printSync(decoded);
printCodegen(parsed);
printBrowser(parsed);

const declaration: AST.TSDeclareFunction = {
  type: "TSDeclareFunction",
  start: 0,
  end: 20,
  id: { type: "Identifier", start: 0, end: 1, name: "f" },
  generator: false,
  async: false,
  params: [],
  typeParameters: null,
  returnType: null,
  declare: true,
  body: null,
  expression: false,
};
printSync({ type: "Program", start: 0, end: 20, hashbang: null, sourceType: "module", body: [declaration] });

const identifier: AST.Identifier = { type: "Identifier", start: 0, end: 1, name: "x" };
const defaultParameter: AST.FormalParameter = {
  type: "AssignmentPattern", start: 0, end: 5, left: identifier, right: identifier,
};
const restParameter: AST.FormalParameterRest = {
  type: "RestElement", start: 0, end: 4, argument: identifier,
};
declaration.params = [defaultParameter, restParameter];
const declarator: AST.VariableDeclarator = {
  type: "VariableDeclarator", start: 0, end: 1, id: identifier, init: null,
};
// @ts-expect-error The receiver is exposed in params, not as an internal thisParam field.
declaration.thisParam;

// Keep useful structure checks on the public AST.
// @ts-expect-error The discriminant is required.
const missingType: Program = { start: 0, end: 0, hashbang: null, body: [], sourceType: "module" };
// @ts-expect-error Statement lists cannot contain primitive values.
printSync({ type: "Program", start: 0, end: 20, hashbang: null, sourceType: "module", body: [42] });
// @ts-expect-error A TS declaration cannot acquire a function body.
declaration.body = { type: "BlockStatement", start: 0, end: 2, body: [] };
for (const statement of parsed.body) {
  if (statement.type === "VariableDeclaration") {
    statement.declarations[0].id;
    // @ts-expect-error Narrowing retains node-specific fields.
    statement.arguments;
  }
}
