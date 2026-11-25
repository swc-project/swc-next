type X<T> = T extends () => (A extends B ? C : (infer U extends string)) ? U : never;
