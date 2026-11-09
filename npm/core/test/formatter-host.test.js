import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { cpSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

test("the formatter host package runs without a language producer", () => {
  const directory = mkdtempSync(join(tmpdir(), "swc-formatter-host-"));
  const source = fileURLToPath(new URL("../../formatter/", import.meta.url));
  try {
    cpSync(join(source, "package.json"), join(directory, "package.json"));
    cpSync(join(source, "src-js"), join(directory, "src-js"), { recursive: true });
    const entry = join(directory, "smoke.mjs");
    writeFileSync(
      entry,
      [
        'import assert from "node:assert/strict";',
        'import * as host from "@swc-next/formatter";',
        'import { encodeDoc } from "@swc-next/formatter/document";',
        'assert.equal(Object.hasOwn(host, "formatDocBufferSync"), false);',
        'const { buffer } = encodeDoc(["prefix", { type: "cursor" }, "λ", { type: "cursor" }, { type: "line", hard: true }, "end"]);',
        'const options = { endOfLine: "crlf" };',
        'const result = host.printDocBufferWithCursorSync(buffer, options);',
        'assert.equal(host.printDocBufferSync(buffer, options), result.formatted);',
        'process.stdout.write(JSON.stringify(result));',
      ].join("\n"),
    );
    const result = JSON.parse(
      execFileSync(process.execPath, [entry], { encoding: "utf8" }),
    );
    assert.deepEqual(result, {
      formatted: "prefixλ\r\nend",
      cursorNodeStart: 6,
      cursorNodeText: "λ",
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
