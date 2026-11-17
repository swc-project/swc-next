export {};
const pick = "outer";
declare namespace N {
  const original: "inner";
  export { original as pick };
}
declare namespace N {
  const result: typeof pick;
}
