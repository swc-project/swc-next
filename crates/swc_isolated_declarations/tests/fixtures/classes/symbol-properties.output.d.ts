// Correct
export declare const foo: {};
export declare abstract class Foo {
  [Symbol.iterator](): void;
  [Symbol.asyncIterator](): Promise<void>;
  [globalThis.Symbol.iterator](): void;
  get [Symbol.toStringTag](): string;
}
// OK because these are not exported
// Incorrect
export declare namespace Foo {
  const foo: {  };
}
export declare function bar(Symbol: {}, globalThis: {}): {};
