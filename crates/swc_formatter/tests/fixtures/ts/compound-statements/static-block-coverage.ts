class Example {
  static // @ts-expect-error
  { missingOutside; }

  static /* Header */ {
    // @ts-expect-error
    missingInside;
  }
}
