declare function f<F>(): void;
type C = typeof f<(| (& (<T>(v: T) => T)))>;
