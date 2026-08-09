# TypeScript Program manifest

`program_manifest.ndjson` records which virtual files the pinned TypeScript
compiler actually includes in each compiler-test configuration. The TypeScript
compiler test runner is the oracle; `.errors.txt` baselines are not used to
infer Program membership.

Each line after the header is one compiler-test configuration. Its compact unit
array contains independent virtual-file occurrences. The header names both
positional schemas and the compact enum values. Unit records have one of three
statuses:

- `e` (exact): the virtual-file occurrence is the exact `SourceFile` used by
  the Program.
- `r` (redirect): TypeScript redirected this path to another package source,
  or a later occurrence shadowed this virtual file.
- `a` (absent): the Program did not contain this virtual file.

The generator uses `SourceFile` identity and `redirectInfo`; source text is
used to verify the canonical host occurrence and detect path shadowing. Every
exact record also stores its UTF-8 byte length and FNV-1a 64-bit hash so a
consumer can prove that it reconstructed the same `SourceFile` text and detect
a stale fixture checkout. At most one exact record may refer to a Program
`SourceFile.path` in a variant. Raw TypeScript module facts are retained so the
consumer does not have to guess grammar context from source text.
Source-file-only columns are `null` for an absent or redirected record.

Install only the small generator dependency, then generate or verify:

```sh
pnpm install
pnpm --filter @swc-next/typescript-program-manifest generate
pnpm --filter @swc-next/typescript-program-manifest verify
```

The generator reads `../fixtures/typescript` at the revision pinned by
`scripts/clone_fixtures.sh`. It creates TypeScript's generated library files in
a temporary directory and never modifies the fixture checkout.
