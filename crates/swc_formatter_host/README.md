# SWC Next Formatter Host

Language-independent formatting sessions for typed source-to-document producers
and encoded documents. Sessions own an arena and shared layout options; language
crates provide parsing, syntax options, and errors. The host has no language
dependencies or runtime plugin loader.

Documents borrow session storage and their inputs. Print or encode them before
releasing either. Use `Writer::import_doc` or `Writer::write_doc` to compose
independent documents with separate named-group scopes.

Run the crate tests with `cargo test -p swc_next_formatter_host`.
