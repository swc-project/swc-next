import fs from "node:fs";
import path from "node:path";

const AST_MARKER = "__ESTREE_TEST__:AST:";
const TOKENS_MARKER = "__ESTREE_TEST__:TOKENS:";

export function readExpected(oracleFile) {
  const contents = fs.readFileSync(oracleFile, "utf8");
  switch (path.extname(oracleFile)) {
    case ".json":
      return [{ ast: contents, tokens: "[]" }];
    case ".md": {
      const asts = readJsonBlocks(contents, AST_MARKER, oracleFile);
      const tokens = readJsonBlocks(contents, TOKENS_MARKER, oracleFile);
      if (asts.length !== tokens.length) {
        throw new Error(`Mismatched AST and token blocks in ${oracleFile}`);
      }
      return asts.map((ast, index) => ({ ast, tokens: tokens[index] }));
    }
    default:
      throw new Error(`Unsupported oracle file: ${oracleFile}`);
  }

  function readJsonBlocks(markdown, marker, file) {
    const blocks = [];
    let searchFrom = 0;

    while (true) {
      const markerIndex = markdown.indexOf(marker, searchFrom);
      if (markerIndex === -1) break;

      const block = markdown
        .slice(markerIndex + marker.length)
        .match(/^\s*```json\r?\n([\s\S]*?)\r?\n```/);
      if (!block) {
        throw new Error(`Invalid JSON block after ${marker} in ${file}`);
      }
      blocks.push(block[1]);
      searchFrom = markerIndex + marker.length + block[0].length;
    }

    if (blocks.length === 0) {
      throw new Error(`Missing ${marker} in ${file}`);
    }

    return blocks;
  }
}
