type Outcome<T> = T extends (() => infer R extends string) ? R : never;
