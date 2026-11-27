import assert from "node:assert/strict";
import test from "node:test";

import swcNextLoader, {
  createLoader,
  SwcNextLoaderError,
} from "@swc-next/loader";

function runLoader(
  source,
  {
    loader = swcNextLoader,
    options,
    resourcePath = "/project/src/index.ts",
    sourceMap = false,
    inputSourceMap,
  } = {},
) {
  let cacheableCalled = false;
  let callbackResult;
  const warnings = [];

  const context = {
    resourcePath,
    sourceMap,
    cacheable() {
      cacheableCalled = true;
    },
    getOptions() {
      return options;
    },
    emitWarning(warning) {
      warnings.push(warning);
    },
    callback(err, code, map) {
      callbackResult = { err, code, map };
    },
  };

  const returnValue = loader.call(context, source, inputSourceMap);
  assert.equal(returnValue, undefined);
  assert.ok(callbackResult, "loader must invoke this.callback");
  return { ...callbackResult, cacheableCalled, warnings };
}

test("transforms TypeScript and TSX using resourcePath, Buffer, and Uint8Array inputs", () => {
  const tsResult = runLoader("export const value: number = 1;", {
    resourcePath: "/project/src/index.ts",
  });
  assert.equal(tsResult.err, null);
  assert.equal(tsResult.cacheableCalled, true);
  assert.equal(tsResult.code, "export const value = 1;\n");
  assert.equal(tsResult.map, undefined);

  const tsxResult = runLoader(
    Buffer.from("export const App = (props: { x: number }) => <div {...props} />;"),
    {
      resourcePath: "/project/src/App.tsx",
      options: {
        transformer: {
          jsx: { runtime: "classic", pragma: "h" },
        },
      },
    },
  );
  assert.equal(tsxResult.err, null);
  assert.match(tsxResult.code, /h\("div", props\)/);

  const bytesResult = runLoader(new TextEncoder().encode("const x: number = 1;"), {
    resourcePath: "/project/src/bytes.ts",
  });
  assert.equal(bytesResult.err, null);
  assert.equal(bytesResult.code, "const x = 1;\n");
});

test("forwards transform options without mutating loader options", () => {
  const options = Object.freeze({
    comments: "none",
    generator: { minify: true },
    transformer: {
      module: { type: "commonjs" },
    },
  });

  const result = runLoader("/* comment */ export const x: number = 1;", {
    resourcePath: "/project/src/mod.ts",
    options,
  });
  assert.equal(result.err, null);
  assert.match(result.code, /Object\.defineProperty\(exports/);
  assert.doesNotMatch(result.code, /comment/);
});

test("handles sourceMap defaults, overrides, and filename", () => {
  const inherited = runLoader("const x = 1;", {
    resourcePath: "/project/src/a.js",
    sourceMap: true,
  });
  assert.equal(inherited.err, null);
  assert.equal(typeof inherited.map, "object");
  assert.equal(inherited.map.version, 3);
  assert.deepEqual(inherited.map.sources, ["/project/src/a.js"]);

  const customFilename = runLoader("const x = 1;", {
    resourcePath: "/project/src/b.js",
    sourceMap: true,
    options: { filename: "custom.js" },
  });
  assert.equal(customFilename.err, null);
  assert.deepEqual(customFilename.map.sources, ["custom.js"]);

  const disabled = runLoader("const x = 1;", {
    resourcePath: "/project/src/c.js",
    sourceMap: true,
    options: { sourceMap: false },
  });
  assert.equal(disabled.err, null);
  assert.equal(disabled.map, undefined);

  const enabled = runLoader("const x = 1;", {
    resourcePath: "/project/src/d.js",
    sourceMap: false,
    options: { sourceMap: true },
  });
  assert.equal(enabled.err, null);
  assert.equal(typeof enabled.map, "object");
  assert.equal(enabled.map.version, 3);
});

test("reports syntax and transform diagnostics as SwcNextLoaderError", () => {
  const syntaxError = runLoader("const a = 1;\nconst = 2;", {
    resourcePath: "/project/src/broken.js",
  });
  assert.ok(syntaxError.err instanceof SwcNextLoaderError);
  assert.match(syntaxError.err.message, /\/project\/src\/broken\.js:2:7:/);
  assert.equal(syntaxError.err.diagnostics.length, 1);
  assert.equal(syntaxError.err.diagnostics[0].severity, "error");

  const invalidTarget = runLoader("const a = 1;", {
    resourcePath: "/project/src/target.js",
    options: { transformer: { target: "not-a-browser 1" } },
  });
  assert.ok(invalidTarget.err instanceof SwcNextLoaderError);
  assert.equal(invalidTarget.err.diagnostics[0].severity, "error");
});

test("supports custom loaders via createLoader", () => {
  const customLoader = createLoader({
    options(options) {
      const { banner: _banner, ...rest } = options;
      return { ...rest, generator: { minify: true } };
    },
    result(result, context) {
      const banner = context.getOptions().banner ?? "";
      return { code: `${banner}${result.code}`, map: result.map };
    },
  });

  const result = runLoader("const value: number = 1;", {
    loader: customLoader,
    resourcePath: "/project/src/custom.ts",
    options: { banner: "/* custom */\n" },
  });
  assert.equal(result.err, null);
  assert.equal(result.code, "/* custom */\nconst value=1;");
});

test("validates source and core options", () => {
  const unknownOption = runLoader("const a = 1;", {
    options: { unknownOption: true },
  });
  assert.ok(unknownOption.err instanceof TypeError);

  const badSource = runLoader(123);
  assert.ok(badSource.err instanceof TypeError);
});
