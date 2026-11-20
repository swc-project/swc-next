type Alias = number;
function Root() {}
namespace Root {
  export function child() {}
  export namespace child {
    export interface Marker {}
  }
}
((Root).child.value) = 1;
namespace Use {
  import Alias = Root.child.value;
  export type T = Alias;
  export const result = Alias;
}
