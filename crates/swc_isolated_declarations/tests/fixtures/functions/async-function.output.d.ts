// Correct
declare function asyncFunctionGood(): Promise<number>;
declare const asyncFunctionGoo2: () => Promise<number>;
declare class AsyncClassGood {
  method(): number;
}
// Need to explicit return type for async functions
// Incorrect
declare function asyncFunction();
declare const asyncFunction2: unknown;
declare class AsyncClassBad {
  method();
}
