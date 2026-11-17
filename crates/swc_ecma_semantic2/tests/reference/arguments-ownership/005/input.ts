declare const arguments: { outer: true };
interface I {
  f(): typeof arguments;
}
