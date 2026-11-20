type U = string;
type R<T> = T extends (
  false extends true ? never : (true extends true ? infer U : never
))
  ? U
  : never;
