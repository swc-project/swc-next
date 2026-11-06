# @swc-next/helpers

Runtime helpers used by SWC Next transforms. Each helper has its own module,
with ESM and CommonJS exports and no runtime dependencies.

```js
import { _to_property_key, _set_function_name } from "@swc-next/helpers";

const key = _to_property_key(Symbol("Widget"));
const Widget = _set_function_name(class {}, key);
```

CommonJS uses the same named exports:

```js
const { _to_property_key } = require("@swc-next/helpers");
```

Install this package alongside code emitted with external helpers. Helpers do
not provide polyfills for built-ins such as `Symbol` or `Object.defineProperty`.

The canonical sources live in `esm/`, one helper per file. Helpers import their
dependencies from sibling modules. After adding a helper, manually regenerate the
entry stubs and CommonJS files with:

```sh
pnpm --filter @swc-next/helpers build
```

The JavaScript script `scripts/build.js` generates explicit named exports for the
package entry point. It uses `@swc/core` to generate CommonJS files; this is only
a development dependency.

The `swc_next_helpers` Rust crate packages the same ESM sources. The transformer
can parse them with its own parser and inject the helpers inline, including
their dependencies, without requiring an npm package at runtime.

Run the public ESM/CommonJS API tests with:

```sh
node --test npm/core/test/helpers.test.js
```
