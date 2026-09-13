## [@napi-rs/cli 3.7.4](./@napi-rs__cli@3.7.4.patch)

The CLI only detects direct `napi-derive` dependencies. Our npm crates use macros
re-exported by `swc_next_bridge`, so this patch enables declaration generation
and cache refresh for crates depending on it.
