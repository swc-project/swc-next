type X<T> = T extends infer U extends A & (B
// c
) ? U : never;
