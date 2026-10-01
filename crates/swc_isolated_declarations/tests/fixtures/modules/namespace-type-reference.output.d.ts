// This ends up being `any`
export type A = Foo.Bar;
declare namespace Foo {
  type Bar = "bar";
}
export {};
