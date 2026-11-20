namespace N {
  export class C {}
}
namespace N {
  export import Value = N.C;
}
import Alias = N.Value;
export { Alias };
