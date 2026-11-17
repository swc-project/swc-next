const key = "other";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Holder {
  export import Item = C.x;
}
namespace Use {
  import A = Holder.Item;
  const result = A;
}
