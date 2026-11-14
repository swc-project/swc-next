function f() {
  // @ts-expect-error
  requestOptions.once = (event: string, handler: (reason: unknown) => void) => {
    return result;
  };
}
