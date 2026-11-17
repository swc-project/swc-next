namespace A {
  export interface T {}
}
namespace V {
  export const x = 1;
}
namespace N {
  export import A = V.x;
}
namespace N {
  import B = A;
  const B = 0;
}
