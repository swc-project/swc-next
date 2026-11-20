export {};
declare namespace N {
  const original: "right";
  export { original as X };
}
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
type Result = typeof N.Inner.Y;
const result: Result = "right";
