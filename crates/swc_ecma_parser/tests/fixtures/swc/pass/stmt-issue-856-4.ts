const _extensions: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    [key: string]: (module: Module, filename: string) => any;
  } = Object.create(null);