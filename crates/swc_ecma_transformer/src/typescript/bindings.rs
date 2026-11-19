use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_semantic2::{Semantic, SymbolFlags, TsAlias};

pub(super) fn resolve_value_binding(semantic: &Semantic<'_>, symbol: SymbolId) -> Option<SymbolId> {
    let mut symbol = semantic.ts_value_binding(symbol);
    loop {
        let flags = semantic.symbol(symbol).flags;
        if flags.in_value_space() {
            return Some(symbol);
        }
        if flags.intersects(SymbolFlags::TYPE_ONLY) {
            return None;
        }
        let Some(TsAlias::Target(target)) = semantic.ts_alias(symbol) else {
            return None;
        };
        symbol = match semantic.ts_export_symbol(target) {
            Some(target) => target,
            None => target,
        }
    }
}
