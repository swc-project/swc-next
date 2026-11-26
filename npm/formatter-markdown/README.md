# @swc-next/formatter-markdown

Native and WebAssembly bindings for the SWC Next Markdown document producer. Both return a SWCD buffer that can be printed directly. For code fences and front matter, the buffer also carries the source ranges and text the Prettier plugin needs to format each embedded block.

```js
import { formatDocSync } from "@swc-next/formatter-markdown";

const { doc, embeds } = formatDocSync("# Hello\n");
```

Pass `{ includeEmbeds: false }` to format the Markdown with its fallback text and omit the embedded block records.

The document format is experimental and unstable. For Markdown source formatting, use the [Prettier plugin](../formatter-prettier-plugin).
