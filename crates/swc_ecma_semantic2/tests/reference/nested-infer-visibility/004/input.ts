type U = string;
type R<T> = T extends (infer U extends unknown ? never : U) ? U : never;
