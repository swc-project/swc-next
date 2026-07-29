import fs from "node:fs";

import {
  CommentMode,
  langFromPath,
  parseSync,
  sourceTypeFromPath,
} from "../src-js/index.js";

const filePath = process.argv[2];
if (!filePath) {
  console.error(
    "Usage: node --experimental-strip-types examples/parser.ts <file>",
  );
  process.exitCode = 1;
} else {
  const source = fs.readFileSync(filePath, "utf8");
  const result = parseSync(source, {
    comments: CommentMode.None,
    lang: langFromPath(filePath),
    preserveParens: true,
    sourceType: sourceTypeFromPath(filePath),
  });

  console.log(
    JSON.stringify(
      result,
      (_key, value) =>
        typeof value === "bigint" ? `${value.toString()}n` : value,
      2,
    ),
  );
}
