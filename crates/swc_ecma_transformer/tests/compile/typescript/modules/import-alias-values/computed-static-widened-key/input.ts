const key: string = "x";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
const Alias = 0;
namespace Use {
  import Alias = C.x;
  export const result: 0 = Alias;
}
