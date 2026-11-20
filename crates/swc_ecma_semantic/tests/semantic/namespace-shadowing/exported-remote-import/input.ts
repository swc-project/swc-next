import * as remote from "./dep.js";
const Alias = { value: "outer" };
namespace N {
  export import Alias = remote;
}
namespace N {
  export const selected = Alias.value;
}
