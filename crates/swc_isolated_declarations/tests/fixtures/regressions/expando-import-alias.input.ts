type Alias = number;

function Root(): void {}
namespace Root {
  export function child(): void {}
  export namespace child {
    export interface Marker {}
  }
}
Root.child.value = 1;

export namespace Use {
  import Alias = Root.child.value;
  export type T = Alias;
}
