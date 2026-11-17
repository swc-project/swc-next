class C {
  static ["x"]() {
    return "x";
  }
}

namespace C {
  export interface x {}
  export interface y {}
}

function A() {
  return 1;
}

function B() {
  return 2;
}

namespace N {
  import A = C.x;
  import B = C.y;
  console.log(A(), B());
}
