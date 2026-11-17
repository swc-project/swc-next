type U = string;
type R<T> = T extends (<V>() => true extends true ? infer U : never) ? U : never;
