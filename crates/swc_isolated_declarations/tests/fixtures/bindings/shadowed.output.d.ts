// Shadowed
export type Foo = {};
export type Bar = {
  foo: Foo;
};
type Func = () => void;
export type FuncType = Func;
type Module = () => void;
declare namespace Module {
  const x = 1;
}
export type ModuleType = Module;
// Regression coverage for shadowed declarations.
// Variable emit should not be dropped when variable is shadowed as a type name
// and the type is part of a class implements interface
export interface Thing<T> {}
declare const Test1: "test1";
export type Test1 = typeof Test1;
export declare class Class1 {
  readonly id: "test1";
}
declare const Test2: "test2";
export type Test2 = typeof Test2;
export declare class Class2 implements Thing<Test2> {
  readonly id: "test2";
}
export {};
