import * as remote from "./dep.js";
const Alias = { value: "outer" };
namespace N {
  import B = remote;
  export import Alias = B;
}
namespace N {
  export const selected = Alias.value;
}
