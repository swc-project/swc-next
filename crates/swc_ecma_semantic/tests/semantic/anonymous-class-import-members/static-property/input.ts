export default interface I {}
export default class {
  static x = 1;
}
export namespace Owner {
  export interface Item {}
}
export { I as Owner };
type A = string;
namespace Test {
  import A = Owner.x;
  export let text: A;
  export const value = A;
}
