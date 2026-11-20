export {};
export default function f(): void;
type f = number;
declare namespace f {
  interface Item {}
}
type A = string;
declare const A: 1;
declare namespace Test {
  import A = f;
  const value: A;
  const scalar: typeof A;
}
