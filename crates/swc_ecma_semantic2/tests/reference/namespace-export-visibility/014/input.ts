export {};
const pick = "outer";
declare namespace N {
  namespace Types {
    interface T {}
  }
  import original = Types.T;
  export { original as pick };
}
namespace N {
  export const result = pick;
}
