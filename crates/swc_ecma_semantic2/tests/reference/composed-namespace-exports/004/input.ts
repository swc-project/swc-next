export {};
declare namespace N {
  export const X: 1;
  interface Original {
    right: true;
  }
  export { Original as X };
}
import Alias = N.X;
const value: 1 = Alias;
const typed: Alias = { right: true };
const queried: typeof Alias = 1;
