import fs from "node:fs";

import {
  langFromPath,
  parseSync,
  sourceTypeFromPath,
} from "@swc-next/core";

const filePath = process.argv[2];

const source = fs.readFileSync(filePath, "utf8");
const result = parseSync(source, {
  comments: "all",
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
