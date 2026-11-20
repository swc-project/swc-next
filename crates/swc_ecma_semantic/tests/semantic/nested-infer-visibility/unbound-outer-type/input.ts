type R<T> = T extends (true extends true ? infer U : never) ? U : never;
