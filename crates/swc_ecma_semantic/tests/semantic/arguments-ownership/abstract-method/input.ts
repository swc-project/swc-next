declare const arguments: { outer: true };
declare abstract class C {
  abstract f(): typeof arguments;
}
