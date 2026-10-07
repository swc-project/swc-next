declare function f<F>(): void;
type C = typeof f</* first */ ((<T>(v: T) => T)) /* last */>;
