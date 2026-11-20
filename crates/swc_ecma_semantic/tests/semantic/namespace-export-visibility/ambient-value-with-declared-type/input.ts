export {};
const pick = "outer";
declare namespace N {
  const original: "inner";
  export { original as pick };
  export interface pick {
    tag: true;
  }
}
declare namespace N {
  const result: typeof pick;
}
