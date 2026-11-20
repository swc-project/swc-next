import * as remote from "./dep.js";
namespace A {
  export interface T {}
}
const B = { value: "outer" };
namespace N {
  import A = remote;
  export import B = A;
}
namespace N {
  export const result = B.value;
}
