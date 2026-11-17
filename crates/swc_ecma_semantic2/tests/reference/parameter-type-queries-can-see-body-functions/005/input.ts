function f({ [("arg" as keyof typeof value)]: arg }: any) {
  function value() {}
}
