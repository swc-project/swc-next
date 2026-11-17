export {};
type Item = { outer: true };
declare namespace N {
  interface Local {
    inner: true;
  }
  export { Local as Item };
}
namespace N {
  export let value: Item;
}
