type R<T> = T extends (boolean extends true ? infer U : infer U) ? U : never;
