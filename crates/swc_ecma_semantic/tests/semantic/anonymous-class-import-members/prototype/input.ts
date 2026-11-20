export default interface I {}
export default class {}
export namespace Owner {
  export interface Item {}
}
export { I as Owner };
type A = string;
namespace Test {
  import A = Owner.prototype;
  export let text: A;
  export const value = A;
}
