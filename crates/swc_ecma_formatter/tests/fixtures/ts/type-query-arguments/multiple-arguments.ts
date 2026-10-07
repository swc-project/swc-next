declare function f<F, G>(): void;
type C = typeof f< <T>(v: T) => T, <U>(v: U) => U>;
