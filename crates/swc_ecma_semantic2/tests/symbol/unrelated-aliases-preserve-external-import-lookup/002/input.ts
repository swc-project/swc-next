import type { Item } from "./dep";
type A = Item;
namespace Unrelated {
  export interface T {}
}
import Unused = Unrelated;
