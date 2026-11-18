use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_semantic2::{Semantic, SymbolFlags, TsAlias};

pub(super) fn value_binding(semantic: &Semantic<'_>, symbol: SymbolId) -> SymbolId {
    if semantic
        .symbol(symbol)
        .flags
        .contains(SymbolFlags::EXPORT_VALUE)
    {
        return semantic.ts_export_symbol(symbol).unwrap();
    }
    symbol
}

pub(super) fn resolve_value_binding(
    semantic: &Semantic<'_>,
    mut symbol: SymbolId,
) -> Option<SymbolId> {
    loop {
        symbol = value_binding(semantic, symbol);
        let flags = semantic.symbol(symbol).flags;
        if flags.in_value_space() {
            return Some(symbol);
        }
        if flags.intersects(SymbolFlags::TYPE_ONLY) {
            return None;
        }
        match semantic.ts_alias(symbol) {
            Some(TsAlias::Target(target)) => symbol = target,
            Some(TsAlias::Unknown) | None => return None,
        }
    }
}
