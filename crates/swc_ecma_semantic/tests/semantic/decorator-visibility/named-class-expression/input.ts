declare function dec<U>(value?: unknown): any;
const C = class Inner {
  @dec<Inner>() m() {}
};
