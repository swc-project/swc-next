export {};
const pick = "outer";
declare namespace N {
  const original: "inner";
  export { original as pick };
  export namespace Inner {}
}
namespace N.Inner {
  export const result = pick;
}
