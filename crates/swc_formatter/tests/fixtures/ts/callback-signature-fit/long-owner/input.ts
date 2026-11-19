export const optionalKey: <A extends AST>(ast: A) => A = memoizeIdempotent(<A extends AST>(ast: A): A => { return ast; });
