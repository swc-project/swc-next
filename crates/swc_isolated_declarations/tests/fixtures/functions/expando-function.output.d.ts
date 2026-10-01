export declare function foo(): void;
export declare const bar: () => void;
export declare namespace NS {
  const goo: () => void;
}
export declare namespace foo {
  // declaration must be exported
  let baz: number;
}
// unexported
declare function qux(): void;
declare namespace qux {
  let woo: number;
}
export default qux;
