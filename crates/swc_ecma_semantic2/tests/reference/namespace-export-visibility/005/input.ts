export {};
type Item = { outer: true };
declare namespace N {
  interface Local {
    inner: true;
  }
  export { Local as Item };
  export const Item: 0;
}
declare namespace N {
  const value: Item;
}
