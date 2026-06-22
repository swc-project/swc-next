import { readFileSync } from "node:fs";
import { parseSync as swcExpParseSync } from "./dist/index.js";
import { parseSync as swcParseSync } from "@swc/core";
import { Bench } from "tinybench";
import { parse as yukuParse } from "yuku-parser";

const sourceUrl = new URL(
  "../../tasks/benchmark/files/angular-1.2.5.js",
  import.meta.url,
);
const source = readFileSync(sourceUrl, "utf8");

let sink;

const swcExpOptions = {};
const yukuOptions = {
  sourceType: "script",
  lang: "js",
  preserveParens: true,
  semanticErrors: false,
  attachComments: false,
};
const swcOptions = {
  syntax: "ecmascript",
  target: "es2022",
  isModule: false,
  comments: false,
};

const bench = new Bench({
  name: "parser benchmark",
  throws: true,
});

bench
  .add("swc-exp parseSync + decode", () => {
    sink = swcExpParseSync(source, swcExpOptions);
  })
  .add("yuku-parser parse + program", () => {
    sink = yukuParse(source, yukuOptions).program;
  })
  .add("@swc/core parseSync", () => {
    sink = swcParseSync(source, swcOptions);
  });

await bench.run();

console.table(bench.table());
