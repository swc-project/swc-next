declare function dec<U>(value?: unknown): any;
class C {
  @(((local: number) => dec(local))(1)) m() {}
}
