declare function dec<U>(value?: unknown): any;
const value = 1;
class C {
  m(@dec(value) value: number) {}
}
