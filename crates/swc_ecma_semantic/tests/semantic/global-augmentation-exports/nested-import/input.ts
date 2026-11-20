export {};
declare namespace N {
  export { X as Y };
}
declare global {
  interface X {}
}
import A = N.Y;
