function outer(first, second) {
  function inner(third) { return first + third + a; }
  function sibling(fourth) { return fourth; }
  return inner(second) + sibling(second);
}
