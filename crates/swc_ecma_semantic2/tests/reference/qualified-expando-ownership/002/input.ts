type Alias = number;
Root.child.leaf["value"] = 1;
function Root() {}
namespace Root {
  export function child() {}
  export namespace child {
    export function leaf() {}
    export namespace leaf {
      export interface Marker {}
    }
  }
}
namespace Use {
  import Alias = Root.child.leaf.value;
  export type T = Alias;
  export const result = Alias;
}
