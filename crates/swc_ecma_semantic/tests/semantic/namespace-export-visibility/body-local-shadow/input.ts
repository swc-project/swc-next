export {};
declare namespace N {
  const original: "inner";
  export { original as pick };
}
namespace N {
  const pick = "local";
  export const result = pick;
}
