export {};
namespace Item {
  export const value = "outer";
}
declare namespace N {
  namespace Local {
    const value: "inner";
  }
  export { Local as Item };
}
namespace N {
  import A = Item;
  export const result = A.value;
}
