declare namespace N {
  namespace A {
    const value: number;
  }
  import Y = A;
  export const X: number;
  export { Y as X };
}
