type Tail<T extends unknown[]> = T extends [unknown, ...infer R] ? R : never;

type Wrapped<T extends unknown[]> = T extends [unknown, ...(((infer R)))] ? R : never;

type Constrained<T extends unknown[]> = T extends [...infer R extends unknown[]] ? R : never;

type Transparent<T extends unknown[]> = T extends [...(| infer R)] ? R : never;
