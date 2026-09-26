declare function asyncFunctionGood(): Promise<number>;
declare const asyncFunctionGoo2: () => Promise<number>;
declare class AsyncClassGood {
	method(): number;
}
declare function asyncFunction();
declare const asyncFunction2: unknown;
declare class AsyncClassBad {
	method();
}
