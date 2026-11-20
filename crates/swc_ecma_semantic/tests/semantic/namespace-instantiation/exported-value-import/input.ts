namespace A {
  export const x = 1;
}
namespace N {
  export import y = A.x;
}
const value = N.y;
