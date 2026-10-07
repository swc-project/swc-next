declare function f<F>(): void;
type C = typeof f<(/* head */ | (<T>(v: T) => T))>;
