type Layout<T extends string, S = unknown> = Reference<T> extends S ? T : S;
