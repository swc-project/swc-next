namespace AliasSpace {
  interface OnlyType {}
  export type Public = OnlyType;
}
namespace AliasSpace {
  export type Good = Public;
  export const bad = Public;
}
