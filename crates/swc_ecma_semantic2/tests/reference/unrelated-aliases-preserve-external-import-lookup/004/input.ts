import { Item } from "./dep";
type A = Item;
const value = Item;
namespace Unrelated {
  export interface T {}
}
import Unused = Unrelated;
