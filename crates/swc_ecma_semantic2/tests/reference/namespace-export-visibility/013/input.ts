export {};
declare namespace N {
  interface Local {
    inner: true;
  }
  export { Local as Item };
  export const Item: 0;
}
namespace N {
  export const result = Item;
}
