type A = string;
namespace V {
  export const x = 1;
}
namespace N {
  export import A = V.x;
}
namespace N {
  export type Result = A;
}
