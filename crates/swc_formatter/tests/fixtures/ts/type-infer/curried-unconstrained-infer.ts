type X<T> = T extends () => () => (infer U) ? U : never;
