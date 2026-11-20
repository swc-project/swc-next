export {};
type U = number;
type R<T> = T extends [infer U extends { value: U }, U] ? U : U;
