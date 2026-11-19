export {};
type U = number;
export type R<T> = T extends [infer U, U] ? U : never;
