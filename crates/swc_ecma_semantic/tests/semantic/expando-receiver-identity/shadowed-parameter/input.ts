export {};
function C() {}
namespace C {
  export interface x {
    own: true;
  }
}
function write(C: { x: number }) {
  C.x = 1;
}
import Alias = C.x;
const typed: Alias = { own: true };
