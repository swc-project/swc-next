# Formatter design

The formatter targets JavaScript, JSX, TypeScript, and TSX using the SWC Next
AST. Compatibility is defined by printed output and deliberate formatting
choices. Prettier 3.9.9 provides the pinned conformance corpus; later fixes are evaluated
separately. Formatting must preserve program semantics.

## Public contract

`format` parses source with flat comment collection and parenthesis preservation,
returning parse errors before formatting. `format_ast` accepts a program AST
whose caller retains its original source, complete comments, and parenthesized
nodes. Fragment roots are outside this contract.

`FormatOptions` is independent of parser options. Native defaults use tabs and a
120-column width; `FormatOptions::prettier` selects Prettier defaults.
Unsupported syntax returns `FormatError::Unsupported` with its source location.

## Responsibilities and data flow

Source or an existing program AST enters syntax formatting, which constructs a
flat document. Printing interprets that document to produce formatted source.

| Layer | Responsibility |
| --- | --- |
| Entry points | Parser prerequisites, options, and errors |
| Syntax formatting | AST syntax, source boundaries, and layout choices |
| Source and comments | Source ranges and comment ownership |
| Document construction | Stable text and a syntax-independent layout protocol |
| Printing | Width, indentation, line endings, and layout selection |

The printer does not interpret AST nodes or recover missing syntax handling.
Document shape need not match Prettier's internal representation; printed output
is the compatibility boundary.

## Verification

Use local fixtures for representative syntax, comments, options, and widths.
The runner checks reparsing, comment retention, and source/AST entry-point
agreement; snapshots expose output and second-pass changes. Parsing and comment
counts alone do not prove semantics: sensitive boundaries need structural or
behavioral checks.

The pinned conformance corpus reports output mismatches, unsupported syntax,
parse errors, panics, and idempotency separately. Unsupported programs remain
visible while syntax support grows.

Performance comparisons use equivalent inputs, options, and builds.
Formatter-only measurements reuse parsed ASTs; source-to-output measurements
include parsing. Instruction counts and elapsed time are evaluated separately.
