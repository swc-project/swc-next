export type R<T> = T extends [infer U, unknown] ? U : never;
