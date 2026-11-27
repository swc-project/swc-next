import assert from "node:assert/strict";
import test from "node:test";

import swcNextLoader, {
  createLoader,
  SwcNextLoaderError,
} from "@swc-next/loader";

function runLoader(
  source,
  { loader = swcNextLoader, options, resourcePath = "/project/src/index.ts", sourceMap = false } = {},
) {
  let cacheableCalled = false;
  let result;

  loader.call(
    {
      resourcePath,
      sourceMap,
      cacheable() {
        cacheableCalled = true;
      },
      getOptions() {
        return options;
      },
      emitWarning() {},
      callback(err, code, map) {
        result = { err, code, map };
      },
    },
    source,
  );

  assert.ok(result, "loader must invoke this.callback");
  return { ...result, cacheableCalled };
}

test("loader transforms source, emits source maps, reports diagnostics, and supports createLoader", () => {
  const transformed = runLoader("export const value: number = 1;", {
    resourcePath: "/project/src/index.ts",
    sourceMap: true,
    options: { comments: "none" },
  });
  assert.equal(transformed.err, null);
  assert.equal(transformed.cacheableCalled, true);
  assert.equal(transformed.code, "export const value = 1;\n");
  assert.equal(transformed.map?.version, 3);
  assert.deepEqual(transformed.map?.sources, ["/project/src/index.ts"]);

  const failed = runLoader("const = 1;", { resourcePath: "/project/src/broken.js" });
  assert.ok(failed.err instanceof SwcNextLoaderError);
  assert.equal(failed.err.diagnostics[0]?.severity, "error");

  const customLoader = createLoader({
    options(options) {
      return { ...options, generator: { minify: true } };
    },
    result(result) {
      return { code: `/* banner */\n${result.code}`, map: result.map };
    },
  });
  const custom = runLoader("const value: number = 1;", { loader: customLoader });
  assert.equal(custom.err, null);
  assert.equal(custom.code, "/* banner */\nconst value=1;");
});
