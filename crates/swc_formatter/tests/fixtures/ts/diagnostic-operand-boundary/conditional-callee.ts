function f() {
  return (c ? a : (// @ts-ignore
  a))(a.p);
}
