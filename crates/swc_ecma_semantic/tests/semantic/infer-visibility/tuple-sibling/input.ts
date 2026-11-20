export {};
type U = number;
type R<T> = T extends [infer U, U] ? U : U;
const ok: R<[string, number]> = "ok";
