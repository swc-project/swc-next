namespace N {
  export interface C {}
}
namespace N {
  export import Value = N.C;
}
import Alias = N.Value;
export { Alias };
