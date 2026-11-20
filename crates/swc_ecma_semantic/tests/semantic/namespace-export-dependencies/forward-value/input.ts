export {};
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
declare namespace N {
  const original: "right";
  export { original as X };
}
type Result = typeof N.Inner.Y;
const result: Result = "right";
