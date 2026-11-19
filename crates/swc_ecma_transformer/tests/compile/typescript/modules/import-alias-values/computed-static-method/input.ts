class C {
  static ["x"]() {
    return 1;
  }
}
namespace C {
  export interface x {}
}
const Alias = 0;
namespace Use {
  import Alias = C.x;
  export const result = Alias();
}
