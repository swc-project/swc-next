# SWC Next Formatter Core

Language-independent document construction and layout printing. Documents use
borrowed text, shared fragments, groups, conditional layouts, Fill and line suffixes.
The formatting arena and input text must outlive printing or document encoding.
Display widths use `unicode-width`; Unicode layout can differ from Prettier.

The document model currently covers the JavaScript and TypeScript formatter's
needs; support for the full Prettier Doc API is still being expanded.

Run the crate tests with `cargo test -p swc_next_formatter_core`.
