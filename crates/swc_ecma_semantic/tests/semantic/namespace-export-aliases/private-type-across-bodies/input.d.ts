declare namespace N {
  interface X {}
  export { X as Y };
}
declare namespace N {
  type T = Y;
}
import A = N.Y;
