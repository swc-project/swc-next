import { readFileSync } from "node:fs";
import { Bench } from "tinybench";
import { parseSync as swcExpParseSync } from "./dist/index.js";
import { parseSync as swcParseSync } from "@swc/core";
import { parse as yukuParse } from "yuku-parser";
import { parse } from "@babel/parser";

const sourceUrl = new URL(
  "../../tasks/benchmark/files/angular-1.2.5.js",
  import.meta.url,
);
const source = readFileSync(sourceUrl, "utf8");

let sink;

const bench = new Bench({
  name: "parser benchmark",
  throws: true,
});

bench
  .add("swc-exp parseSync", () => {
    sink = swcExpParseSync(source, {});
  })
  .add("yuku-parser parse", () => {
    sink = yukuParse(source, {
      sourceType: "script",
      lang: "js",
      preserveParens: true,
      semanticErrors: false,
      attachComments: false,
    });
  })
  .add("@swc/core parseSync", () => {
    sink = swcParseSync(source, {
      syntax: "ecmascript",
      target: "es2022",
      isModule: false,
      comments: false,
    });
  })
  .add("@babel/parser parse", () => {
    sink = parse(source, {
      sourceType: "module",
      attachComment: false,
      tokens: false,
      errorRecovery: false,
    });
  });

await bench.run();

console.table(bench.table());
