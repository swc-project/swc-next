namespace N {
  export import A0 = N.A1;
  export namespace A1 { export interface T {} }
}
import Root = N.A0;
type Result = Root.T;
