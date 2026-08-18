import fs from "node:fs";

import {
  langFromPath,
  parseSync,
  sourceTypeFromPath,
} from "../src-js/index.js";

const filePath = process.argv[2];

const source = fs.readFileSync(filePath, "utf8");
const result = parseSync(source, {
  comments: "flat",
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
