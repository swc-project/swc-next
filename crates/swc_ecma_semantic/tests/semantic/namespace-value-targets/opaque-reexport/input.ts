import * as remote from "pkg";
declare namespace N {
  import original = remote.value;
  export { original as value };
  export type { original as erased };
}
namespace N {
  export const result = value;
}
