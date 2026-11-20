import * as Item from "./dep";
type A = typeof Item;
namespace Unrelated {
  export interface T {}
}
import Unused = Unrelated;
