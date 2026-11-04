type X<T> = {
  [K in keyof T]: T[K]
  // trailing own-line comment
};
