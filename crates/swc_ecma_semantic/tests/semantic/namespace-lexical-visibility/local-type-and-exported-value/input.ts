namespace N {
  export interface X {
    exported: true;
  }
}
namespace N {
  interface X {
    local: true;
  }
  export const X = 1;
  type T = X;
}
