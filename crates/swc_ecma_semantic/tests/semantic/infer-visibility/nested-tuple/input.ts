export {};
type U = number;
type R<T> = T extends [infer U, T extends [infer V, U] ? U : U] ? U : U;
