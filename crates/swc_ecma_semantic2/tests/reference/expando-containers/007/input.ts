function Root() {}
namespace Root {
  export function child() {}
  export namespace child {
    export interface Marker {}
  }
}
Root["child"].value = 1;
import Alias = Root.child.value;
type T = Alias;
