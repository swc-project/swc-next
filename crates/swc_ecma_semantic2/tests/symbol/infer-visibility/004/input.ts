export {};
type U = number;
type R<T> = T extends infer U ? (U extends [infer V, U] ? U : U) : U;
