# @swc-next/helpers

Runtime helpers used by SWC Next transforms. Each helper has its own ESM module,
with no runtime dependencies.

```js
import { _to_property_key, _set_function_name } from "@swc-next/helpers";

const key = _to_property_key(Symbol("Widget"));
const Widget = _set_function_name(class {}, key);
```

Install this package alongside code emitted with external helpers. Helpers do
not provide polyfills for built-ins such as `Symbol` or `Object.defineProperty`.

The canonical sources live in `esm/`, one helper per file. Helpers import their
dependencies from sibling modules. After adding a helper, manually regenerate the
entry stub with:

```sh
pnpm --filter @swc-next/helpers build
```

The script generates explicit named exports for the package entry point.

Scripts and CommonJS output load named exports with `require`. This requires a
runtime that supports requiring ES modules, or a bundler that handles them.

Run the public ESM API tests with:

```sh
node --test npm/core/test/helpers.test.js
```
