const X = 1;
declare namespace N {
  export interface X {}
  const value: 1;
  export type { value as X };
  export { X as Y };
}
