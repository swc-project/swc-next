export {};
const pick = "outer";
declare namespace N {
  const original: "inner";
  export { original as pick };
  export interface pick {
    tag: true;
  }
}
namespace N {
  export const result = pick;
}
