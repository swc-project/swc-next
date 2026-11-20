export {};
declare namespace N {
  export interface X {
    right: true;
  }
  const original: 1;
  export { original as X };
}
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
namespace N.Inner {
  export const value: 1 = Y;
  export const typed: Y = { right: true };
  export const queried: typeof Y = 1;
}
