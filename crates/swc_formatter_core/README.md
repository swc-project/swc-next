# swc_formatter_core

Language-independent document construction and layout printing for SWC Next. Supports groups, indentation, conditional layouts, line suffixes, and document encoding.

**Experimental and unstable.** APIs, document formats, and output may change. Unicode width measurement can produce different line breaks from Prettier.

## Usage

```rust
use swc_next_allocator::Allocator;
use swc_next_formatter_core::{PrintOptions, Writer};

let allocator = Allocator::new();
let mut writer = Writer::new(&allocator);
writer.text("hello");
writer.hard_line();
writer.text("world");
let output = writer.finish().print(PrintOptions::default());
```

See the [crate API](src/lib.rs) for document operations and storage requirements. For JavaScript and TypeScript source formatting, use [swc_formatter_js](../swc_formatter_js).

## Testing

Unit tests cover document construction, layout, and binary encoding and decoding.

```sh
cargo test -p swc_next_formatter_core --locked
```
