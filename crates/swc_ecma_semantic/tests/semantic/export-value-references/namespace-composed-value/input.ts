declare namespace N {
  export interface X {}
  const value: 1;
  export { value as X };
  export { X as Y };
}
