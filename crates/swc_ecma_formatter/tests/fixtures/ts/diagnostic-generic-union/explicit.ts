function f(
  // @ts-expect-error
  a: A<| B<C> | null> = d,
) {}
