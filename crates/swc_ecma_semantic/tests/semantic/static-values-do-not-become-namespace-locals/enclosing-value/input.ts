export {};
const x = 2;
class C {
  static x = 1;
}
namespace C {
  export const y = x;
}
