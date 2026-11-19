type U = number;
export type R<T> = T extends [infer U, unknown] ? U : U;
export {};
