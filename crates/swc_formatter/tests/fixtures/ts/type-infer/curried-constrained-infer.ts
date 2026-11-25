type X<T> = T extends () => new () => (x: any) => x is (infer U extends string) ? U : never;
