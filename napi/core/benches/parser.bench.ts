import fs from "node:fs/promises";
import path from "node:path";
import { Bench } from "tinybench";

import { parse as babelParse, type ParserPlugin } from "@babel/parser";
import {
  parseSync as swcParse,
  type ParseOptions as SwcParseOptions,
} from "@swc/core";
import {
  langFromPath as yukuLangFromPath,
  parse as yukuParse,
  sourceTypeFromPath as yukuSourceTypeFromPath,
} from "yuku-parser";
import {
  CommentMode,
  Lang,
  SourceType,
  langFromPath,
  parseSync,
  sourceTypeFromPath,
} from "../src-js/index.js";

const BENCHES = path.resolve(
  import.meta.dirname,
  "../../../tasks/benchmark/files",
);

for (const bench of await fs.readdir(BENCHES, { withFileTypes: true })) {
  const inputPath = path.join(BENCHES, bench.name);
  await runBenchmark(inputPath);
}

async function runBenchmark(inputPath: string): Promise<void> {
  const source = await fs.readFile(inputPath, "utf8");
  const lang = langFromPath(inputPath);
  const sourceType = sourceTypeFromPath(inputPath);
  const isTypeScript =
    lang === Lang.Ts || lang === Lang.Tsx || lang === Lang.Dts;
  const hasJsx = lang === Lang.Jsx || lang === Lang.Tsx;

  const swcOptions: SwcParseOptions = isTypeScript
    ? {
        syntax: "typescript",
        tsx: hasJsx,
        decorators: true,
        target: "esnext",
        comments: false,
        script: sourceType !== SourceType.Module,
      }
    : {
        syntax: "ecmascript",
        jsx: hasJsx,
        decorators: true,
        target: "esnext",
        comments: false,
        script: sourceType !== SourceType.Module,
      };

  const babelPlugins: ParserPlugin[] = [];
  if (isTypeScript) {
    babelPlugins.push(["typescript", { dts: lang === Lang.Dts }]);
  }
  if (hasJsx) {
    babelPlugins.push("jsx");
  }

  const currentOptions = {
    comments: CommentMode.None,
    lang,
    preserveParens: true,
    sourceType,
  };

  const yukuOptions = {
    attachComments: false,
    lang: yukuLangFromPath(inputPath),
    preserveParens: true,
    semanticErrors: false,
    sourceType: yukuSourceTypeFromPath(inputPath),
  };

  const babelOptions = {
    attachComment: false,
    createParenthesizedExpressions: true,
    errorRecovery: true,
    plugins: babelPlugins,
    sourceType:
      sourceType === SourceType.Module
        ? ("module" as const)
        : ("script" as const),
  };

  // Load native bindings and validate all configurations before timing them.
  parseSync(source, currentOptions);
  yukuParse(source, yukuOptions).program;
  babelParse(source, babelOptions);
  swcParse(source, swcOptions);

  const bench = new Bench({
    iterations: 1,
    time: 1_000,
    warmup: true,
    warmupIterations: 1,
    warmupTime: 250,
  });

  bench
    .add("@swc-next/core.parseSync", () => {
      parseSync(source, currentOptions);
    })
    .add("yuku-parser", () => {
      yukuParse(source, yukuOptions).program;
    })
    .add("@babel/parser", () => {
      babelParse(source, babelOptions);
    })
    .add("@swc/core.parseSync", () => {
      swcParse(source, swcOptions);
    });

  await bench.run();
  console.table(bench.table());
}
