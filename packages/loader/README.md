# `@swc-next/loader`

Webpack and Rspack loader powered by `@swc-next/core`.

## Usage

```js
// webpack.config.js / rspack.config.js
export default {
  module: {
    rules: [
      {
        test: /\.[cm]?[jt]sx?$/,
        exclude: /node_modules/,
        use: {
          loader: "@swc-next/loader",
          options: {
            transformer: {
              target: "es2022",
            },
          },
        },
      },
    ],
  },
};
```

## Options

`@swc-next/loader` accepts all `TransformApiOptions` from `@swc-next/core`.

When omitted:

- `filename` defaults to the loader's `resourcePath`, which `@swc-next/core` uses to infer language and source type.
- `sourceMap` defaults to the bundler's `this.sourceMap` flag. Generated source maps are passed to the bundler callback as parsed Source Map v3 objects.

## Customization

Use `createLoader` to wrap `@swc-next/loader` with custom per-file option resolution or output post-processing:

```js
import { createLoader } from "@swc-next/loader";

export default createLoader({
  options(options, context) {
    return { ...options, comments: "none" };
  },
});
```

## Limitations

- Execution is synchronous (`transformSync`); async `transform` and a `sync` toggle are not yet available.
- Chained `inputSourceMap` merging is not yet supported by `@swc-next/core`.

## Testing

```sh
pnpm --filter @swc-next/loader test
```
