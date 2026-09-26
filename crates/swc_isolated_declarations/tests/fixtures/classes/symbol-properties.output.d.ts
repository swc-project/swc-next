// Correct
export declare const foo: {};
export declare abstract class Foo {
    [Symbol.iterator](): void;
    [Symbol.asyncIterator](): Promise<void>;
    [globalThis.Symbol.iterator](): void;
    get [Symbol.toStringTag](): string;
}
// Incorrect
export declare namespace Foo {
    const foo: {
    };
}
export declare function bar(Symbol: {
}, globalThis: {
}): {
};
