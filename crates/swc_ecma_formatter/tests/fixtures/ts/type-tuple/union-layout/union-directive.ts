type Value = [
  number,
  // @ts-expect-error
  | MissingType | "alpha" | "beta" | "gamma",
];
