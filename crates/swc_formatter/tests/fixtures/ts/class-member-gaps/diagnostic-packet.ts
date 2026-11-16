class Value {
  first() {}

  // ordinary head
  // @ts-expect-error
  protectedValue = MissingOne + MissingTwo;
  outsideValue = OutsideValue;
}
