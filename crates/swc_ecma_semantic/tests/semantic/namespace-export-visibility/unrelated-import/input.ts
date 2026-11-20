export {};
const pick = "outer";
declare namespace N {
  const original: "inner";
  export { original as pick };
}
namespace N {
  export const result = pick;
}
namespace Unrelated {}
import Unused = Unrelated;
