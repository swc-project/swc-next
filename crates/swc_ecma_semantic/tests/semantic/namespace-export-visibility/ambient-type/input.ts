export {};
type Item = { outer: true };
declare namespace N {
  interface Local {
    inner: true;
  }
  export { Local as Item };
}
declare namespace N {
  const value: Item;
}
