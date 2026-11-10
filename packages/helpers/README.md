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

The `swc_next_helpers` Rust crate packages the same ESM sources. The transformer
can parse them with its own parser and inject the helpers inline, including
their dependencies, without requiring an npm package at runtime.

Run the public ESM API tests with:

```sh
node --test npm/core/test/helpers.test.js
```

## Third-party notices

`_to_property_key`, `_to_primitive`, and `_type_of` are adapted from
[SWC helpers](https://github.com/swc-project/swc/tree/main/packages/helpers/esm),
licensed under Apache-2.0.

`_set_function_name` is adapted from
[Babel helpers](https://github.com/babel/babel/blob/main/packages/babel-helpers/src/helpers/setFunctionName.ts),
under the following license:

```text
MIT License

Copyright (c) 2014-present Sebastian McKenzie and other contributors

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
