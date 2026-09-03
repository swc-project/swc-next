import fs from "node:fs/promises";
import path from "node:path";
import { Bench } from "tinybench";

import { parse as babelParse, type ParserPlugin } from "@babel/parser";
import {
  parseSync as swcParse,
  type ParseOptions as SwcParseOptions,
} from "@swc/core";
import {
  parseSync as oxcParse,
  rawTransferSupported as oxcRawTransferSupported,
  type ParserOptions as OxcParserOptions,
} from "oxc-parser";
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

  const oxcOptions: OxcParserOptions = {
    lang:
      lang === Lang.Dts
        ? "dts"
        : isTypeScript
          ? hasJsx
            ? "tsx"
            : "ts"
          : hasJsx
            ? "jsx"
            : "js",
    preserveParens: true,
    showSemanticErrors: false,
    sourceType:
      sourceType === SourceType.Module
        ? ("module" as const)
        : ("script" as const),
  };
  const oxcRawTransferOptions: OxcParserOptions & {
    experimentalRawTransfer: true;
  } = {
    ...oxcOptions,
    experimentalRawTransfer: true,
  };
  const hasOxcRawTransfer = oxcRawTransferSupported();

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
  parseSync(source, currentOptions).program;
  yukuParse(source, yukuOptions).program;
  babelParse(source, babelOptions);
  swcParse(source, swcOptions);
  oxcParse(inputPath, source, oxcOptions).program;
  if (hasOxcRawTransfer) {
    oxcParse(inputPath, source, oxcRawTransferOptions).program;
  }

  const bench = new Bench({
    iterations: 1,
    time: 1_000,
    warmup: true,
    warmupIterations: 1,
    warmupTime: 250,
  });

  bench
    .add("@swc-next/core", () => {
      parseSync(source, currentOptions).program;
    })
    .add("yuku-parser", () => {
      yukuParse(source, yukuOptions).program;
    })
    .add("@babel/parser", () => {
      babelParse(source, babelOptions);
    })
    .add("@swc/core", () => {
      swcParse(source, swcOptions);
    })
    .add("oxc-parser", () => {
      oxcParse(inputPath, source, oxcOptions).program;
    });
  if (hasOxcRawTransfer) {
    bench.add("oxc-parser (raw transfer)", () => {
      oxcParse(inputPath, source, oxcRawTransferOptions).program;
    });
  }

  await bench.run();
  console.table(bench.table());
}
