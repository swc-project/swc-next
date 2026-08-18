import fs from "node:fs/promises";
import path from "node:path";
import { Bench } from "tinybench";

import {
  CommentMode,
  langFromPath,
  parseSync,
  sourceTypeFromPath,
} from "../src-js/index.js";

const FIXTURES = path.resolve(
  import.meta.dirname,
  "../../../tasks/benchmark/files"
);

for (const fixture of await fs.readdir(FIXTURES, { withFileTypes: true })) {
  if (!fixture.isFile()) {
    continue;
  }

  const inputPath = path.join(FIXTURES, fixture.name);
  await runBenchmark(inputPath);
}

async function runBenchmark(inputPath: string): Promise<void> {
  const source = await fs.readFile(inputPath, "utf8");
  const options = {
    comments: CommentMode.None,
    lang: langFromPath(inputPath),
    preserveParens: true,
    sourceType: sourceTypeFromPath(inputPath),
  };
  const flatOptions = {
    ...options,
    comments: CommentMode.Flat,
  };

  // Load the native binding and exercise every path before measuring it.
  parseSync(source, options);
  parseSync(source, flatOptions);
  parseSync(source, flatOptions).comments.length;

  const bench = new Bench({
    iterations: 1,
    time: 1_000,
    warmup: true,
    warmupIterations: 1,
    warmupTime: 250,
  });

  bench
    .add("comments: none", () => {
      parseSync(source, options);
    })
    .add("comments: flat (lazy)", () => {
      parseSync(source, flatOptions);
    })
    .add("comments: flat (materialized)", () => {
      parseSync(source, flatOptions).comments.length;
    });

  await bench.run();
  console.log(path.basename(inputPath));
  console.table(bench.table());
}
