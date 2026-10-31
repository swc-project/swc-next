type Elements<T> = T extends [unknown, ...(infer R)[]] ? R : never;

type Union<T extends unknown[], U extends unknown[]> = [...(T | U)];

type Intersection<T extends unknown[], U extends unknown[]> = [...(T & U)];

type Readonly<T extends unknown[]> = [...(readonly T[])];

type Callable = [...(() => unknown[])];

type Optional<T extends unknown[]> = [...(T[])?];
