type U = string;
type R<T> = T extends { [K in "x"]: true extends true ? infer U : never }
  ? U
  : never;
