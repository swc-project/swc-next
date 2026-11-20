const Alias = 0;
namespace N {
  const fn = () => {};
  namespace fn {
    export interface x {}
  }
  fn.x = 1;
  import Alias = fn.x;
  export const result = Alias;
}
